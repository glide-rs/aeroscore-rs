use failure::Error;
use flat_projection::FlatPoint;
use log::{debug, trace};
use ord_subset::OrdVar;

use crate::Point;
use crate::flat::to_flat_points;
use crate::haversine::haversine_distance;
use crate::parallel::*;

const LEGS: usize = 6;

pub type Path = Vec<usize>;

#[derive(Debug)]
pub struct OptimizationResult {
    pub path: Path,
    pub distance: f32,
}

pub fn optimize<T: Point>(route: &[T]) -> Result<OptimizationResult, Error> {
    debug!("Converting {} points to flat points", route.len());
    let flat_points = to_flat_points(route);

    debug!("Calculating distance matrix (finish -> start)");
    let dist_matrix = half_dist_matrix(&flat_points);

    debug!("Calculating solution graph");
    let graph = Graph::from_distance_matrix(&dist_matrix);

    debug!("Searching for best valid solution");
    let mut best_valid = graph.find_best_valid_solution(&route);
    debug!("-- New best solution: {:.3} km -> {:?}", calculate_distance(route, &best_valid.path), best_valid.path);

    debug!("Searching for potentially better solutions");
    let mut end_candidates: Vec<_> = graph.g[LEGS - 1].iter()
        .enumerate()
        .filter(|(_, cell)| cell.distance > best_valid.distance)
        .map(|(start_index, cell)| StartCandidate { distance: cell.distance, start_index })
        .collect();

    end_candidates.sort_by_key(|it| OrdVar::new_checked(it.distance));
    debug!("{} potentially better start points found", end_candidates.len());

    while let Some(candidate) = end_candidates.pop() {
        let finish_altitude = route[candidate.start_index].altitude();

        debug!("Calculating solution graph with start point at index {}", candidate.start_index);
        let candidate_graph = Graph::for_start_index(finish_altitude, &dist_matrix, route);
        let best_valid_for_candidate = candidate_graph.find_best_valid_solution(&route);
        if best_valid_for_candidate.distance > best_valid.distance {
            best_valid = best_valid_for_candidate;
            debug!("-- New best solution: {:.3} km -> {:?}", calculate_distance(route, &best_valid.path), best_valid.path);

            end_candidates.retain(|it| it.distance > best_valid.distance);
        } else {
            debug!("Discarding solution with {:.3} km", calculate_distance(route, &best_valid_for_candidate.path));
        }

        debug!("{} potentially better end points left", end_candidates.len());
    }

    let distance = calculate_distance(route, &best_valid.path);
    debug!("Solution: {:?} ({:.3} km)", best_valid.path, distance);

    Ok(OptimizationResult { distance, path: best_valid.path })
}

#[derive(Debug)]
struct StartCandidate {
    distance: f32,
    start_index: usize,
}

/// Generates a triangular matrix with the distances in kilometers between all points.
/// For each point, the distance to the preceding points is saved. This only allows
/// calculation of forward min-marginals
pub fn half_dist_matrix(flat_points: &[FlatPoint<f32>]) -> Vec<Vec<f32>> {
    opt_par_iter(flat_points)
        .enumerate()
        .map(|(i, p1)| flat_points[..i+1].iter()
            .map(|p2| p1.distance(p2))
            .collect()) 
        .collect()
}

struct Graph {
    g: Vec<Vec<GraphCell>>,
}

#[derive(Debug)]
struct GraphCell {
    prev_index: usize,
    distance: f32,
}

impl Graph {
    fn from_distance_matrix(dist_matrix: &[Vec<f32>]) -> Self {
        let mut graph: Vec<Vec<GraphCell>> = Vec::with_capacity(LEGS);

        trace!("-- Analyzing leg #{}", 1);

        let layer: Vec<GraphCell> = opt_par_iter(dist_matrix)
            .map(|distances| distances.iter()
                .enumerate()
                .map(|(start_index, &distance)| GraphCell { prev_index: start_index, distance })
                .max_by_key(|cell| OrdVar::new_checked(cell.distance))
                .unwrap())
            .collect();

        graph.push(layer);

        for layer_index in 1..LEGS {
            trace!("-- Analyzing leg #{}", layer_index+1);
            let last_layer = &graph[layer_index - 1];

            let layer: Vec<GraphCell> = opt_par_iter(dist_matrix)
                .map(|distances| {
                    distances.iter()
                        .zip(last_layer.iter())
                        .enumerate()
                        .map(|(prev_index, (&leg_dist, last_layer_cell))| {
                            let distance = last_layer_cell.distance + leg_dist;
                            GraphCell { prev_index: prev_index, distance }
                        })
                        .max_by_key(|cell| OrdVar::new_checked(cell.distance))
                        .unwrap()
                })
                .collect();

            graph.push(layer);
        }

        Graph { g: graph }
    }

    fn for_start_index<T: Point>(finish_altitude: i16, dist_matrix: &[Vec<f32>], points: &[T]) -> Self {
        let mut graph: Vec<Vec<GraphCell>> = Vec::with_capacity(LEGS);

        trace!("-- Analyzing leg #{}", 1);

        // Only use start points which are compliant to 1000m rule
        //
        // assuming X is the first turnpoint, what is the distance to `start_index`?

        let layer: Vec<GraphCell> = opt_par_iter(dist_matrix)
            .map(|distances| distances.iter()
                .enumerate()
                .map(|(start_index, &distance)| {
                    let start = &points[start_index];
                    let altitude_delta = start.altitude() - finish_altitude;
                    if altitude_delta <= 1000  {
                        GraphCell { prev_index: start_index, distance: distance }
                    } else {
                        GraphCell { prev_index: start_index, distance: distance - 100_000.0}
                    }
                })
                .max_by_key(|cell| OrdVar::new_checked(cell.distance))
                .unwrap())
            .collect();
        // trace!("Layer: {:?}", layer);
        graph.push(layer);

        for layer_index in 1..LEGS {
            trace!("-- Analyzing leg #{}", layer_index + 1);

            // layer: 1 / leg: 2
            //
            // assuming X is the second turnpoint, what is the first turnpoint
            // that results in the highest total distance?
            //
            // layer: 2 / leg: 3
            //
            // assuming X is the third turnpoint, what is the second turnpoint
            // that results in the highest total distance?
            //
            // ...
            //
            // layer: 5 / leg: 6
            //
            // assuming X is the finish point, what is the fifth turnpoint
            // that results in the highest total distance?

            let last_layer = &graph[layer_index - 1];
            let layer: Vec<GraphCell> = opt_par_iter(dist_matrix)
                .map(|distances| {
                    distances.iter()
                        .zip(last_layer.iter())
                        .enumerate()
                        .map(|(prev_index, (&leg_dist, last_layer_cell))| {
                            let distance = last_layer_cell.distance + leg_dist;
                            GraphCell { prev_index, distance }
                        })
                        .max_by_key(|cell| OrdVar::new_checked(cell.distance))
                        .unwrap()
                })
                .collect();
            graph.push(layer);
        }

        Graph { g: graph }
    }

    /// Finds the best (largest distance), valid (with 1000m rule) path
    /// through the graph and returns `(distance, path)`
    fn find_best_valid_solution<T: Point>(&self, points: &[T]) -> OptimizationResult {
        let last_graph_row = self.g.last().unwrap();

        let offset = points.len() - last_graph_row.len();

        last_graph_row.iter()
            .enumerate()
            .filter_map(|(index, cell)| {
                let iter = GraphIterator {
                    graph: self,
                    next: Some((self.g.len(), index + offset)),
                    offset,
                };

                let mut path = iter.collect::<Vec<_>>();
                if *path.first().unwrap() > *path.last().unwrap() {
                    path.reverse();
                }

                let start_index = *path.first().unwrap();
                let finish_index = *path.last().unwrap();
                let start = &points[start_index];
                let finish = &points[finish_index];
                let altitude_delta = start.altitude() - finish.altitude();
                if altitude_delta <= 1000  {
                    Some(OptimizationResult { distance: cell.distance, path })
                } else {
                    None
                }
            })
            .max_by_key(|result| OrdVar::new_checked(result.distance))
            .unwrap()
    }
}

struct GraphIterator<'a> {
    graph: &'a Graph,
    next: Option<(usize, usize)>,
    offset: usize,
}

impl Iterator for GraphIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_none() { return None; }

        let (layer, index) = self.next.unwrap();
        self.next = if layer == 0 {
            None
        } else {
            let next_layer = layer - 1;
            let next_index = self.graph.g[next_layer][index - self.offset].prev_index;
            Some((next_layer, next_index))
        };

        Some(index)
    }
}

/// Calculates the total task distance (via haversine algorithm) from
/// the original `route` and the array of indices
///
fn calculate_distance<T: Point>(points: &[T], path: &Path) -> f32 {
    path.iter().zip(path.iter().skip(1))
        .map(|(i1, i2)| (&points[*i1], &points[*i2]))
        .map(|(fix1, fix2)| haversine_distance(fix1, fix2))
        .sum()
}