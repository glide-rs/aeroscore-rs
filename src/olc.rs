use failure::Error;
use flat_projection::FlatPoint;
use log::debug;
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
    let dist_matrix = full_dist_matrix(&flat_points);

    debug!("Calculating solution graph");
    let graph = Graph::from_distance_matrix(&dist_matrix);

    debug!("Searching for best valid solution");
    let mut best_valid = graph.find_best_valid_solution(&route);
    debug!("Found best valid solution: {:?} ({:.3} km)", best_valid.path, calculate_distance(route, &best_valid.path));

    debug!("Searching for potentially better solutions");
    let mut start_candidates: Vec<_> = graph.g[LEGS - 1].iter()
        .enumerate()
        .filter(|(_, cell)| cell.distance > best_valid.distance)
        .map(|(start_index, cell)| StartCandidate { distance: cell.distance, start_index })
        .collect();

    start_candidates.sort_by_key(|it| OrdVar::new_checked(it.distance));

    while let Some(candidate) = start_candidates.pop() {
        debug!("{} potentially better start points left", start_candidates.len());

        debug!("Calculating solution graph with start point at index {}", candidate.start_index);
        let candidate_graph = Graph::for_start_index(candidate.start_index, &dist_matrix);

        let best_valid_for_candidate = candidate_graph.find_best_valid_solution(&route);
        debug!("Found best valid solution for start point at index {}: {:?} ({:.3} km)", candidate.start_index, best_valid_for_candidate.path, calculate_distance(route, &best_valid_for_candidate.path));

        if best_valid_for_candidate.distance > best_valid.distance {
            debug!("New best solution: {:.3} km", calculate_distance(route, &best_valid_for_candidate.path));
            best_valid = best_valid_for_candidate;
        }

        start_candidates.retain(|it| it.distance > best_valid.distance);
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

/// Generates a N*N matrix with the distances in kilometers between all points.
fn full_dist_matrix(flat_points: &[FlatPoint<f32>]) -> Vec<Vec<f32>> {
    opt_par_iter(flat_points)
        .map(|p1| flat_points.iter()
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

        // layer: 0 / leg: 6
        //
        // assuming X is the fifth turnpoint, what is the furthest away finish point
        debug!("-- Analyzing leg #{}", 6);

        let layer = opt_par_iter(dist_matrix)
            .enumerate()
            .map(|(tp_index, distances)| distances.iter()
                .enumerate()
                .skip(tp_index)
                .map(|(finish_index, &distance)| GraphCell { prev_index: finish_index, distance })
                .max_by_key(|cell| OrdVar::new_checked(cell.distance))
                .unwrap())
            .collect();

        graph.push(layer);

        for layer_index in 1..LEGS {
            debug!("-- Analyzing leg #{}", LEGS - layer_index);

            // layer: 1 / leg: 5
            //
            // assuming X is the fourth turnpoint, what is the fifth turnpoint
            // that results in the highest total distance?
            //
            // layer: 2 / leg: 4
            //
            // assuming X is the third turnpoint, what is the fourth turnpoint
            // that results in the highest total distance?
            //
            // ...
            //
            // layer: 5 / leg: 1
            //
            // assuming X is the start point, what is the first turnpoint
            // that results in the highest total distance?

            let last_layer = &graph[layer_index - 1];

            let layer = opt_par_iter(dist_matrix)
                .enumerate()
                .map(|(tp_index, distances)| {
                    distances.iter()
                        .zip(last_layer.iter())
                        .enumerate()
                        .skip(tp_index)
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

    fn for_start_index(start_index: usize, dist_matrix: &[Vec<f32>]) -> Self {
        let mut graph: Vec<Vec<GraphCell>> = Vec::with_capacity(LEGS);

        debug!("-- Analyzing leg #{}", 1);

        // layer: 0 / leg: 1
        //
        // assuming X is the first turnpoint, what is the distance to `start_index`?
        let layer = dist_matrix[start_index].iter()
            // skip points before start_index
            .skip(start_index)
            .map(|&distance| GraphCell { prev_index: start_index, distance })
            .collect();

        graph.push(layer);

        for layer_index in 1..LEGS {
            debug!("-- Analyzing leg #{}", layer_index + 1);

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

            let layer = opt_par_iter(dist_matrix)
                // skip points before start_index
                .skip(start_index)
                .enumerate()
                .map(|(tp_index, distances)| {
                    distances.iter()
                        .skip(start_index)
                        .zip(last_layer.iter())
                        .enumerate()
                        .take(tp_index + 1)
                        .map(|(prev_index, (&leg_dist, last_layer_cell))| {
                            let prev_index = prev_index + start_index;
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
/// the original `route` and the arry of indices
///
fn calculate_distance<T: Point>(points: &[T], path: &Path) -> f32 {
    path.iter().zip(path.iter().skip(1))
        .map(|(i1, i2)| (&points[*i1], &points[*i2]))
        .map(|(fix1, fix2)| haversine_distance(fix1, fix2))
        .sum()
}
