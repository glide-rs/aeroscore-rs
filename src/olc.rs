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
    pub point_list: Path,
    pub distance: f32,
}

pub fn optimize<T: Point>(route: &[T]) -> Result<OptimizationResult, Error> {
    debug!("Converting {} points to flat points", route.len());
    let flat_points = to_flat_points(route);

    debug!("Calculating distance matrix");
    let distance_matrix = following_dist_matrix(&flat_points);

    debug!("Calculating solution graph");
    let graph = Graph::from_distance_matrix(&distance_matrix);

    debug!("Searching for best solution");
    let path = graph.find_max_distance_path();
    debug!("Found best solution: {:?}", path);

    let altitude_delta = find_altitude_delta(&path, &route);
    debug!("Best solution altitude difference: {:?}m", altitude_delta);

    let distance = calculate_distance(route, &path);
    debug!("Distance for best solution: {} km", distance);

    Ok(OptimizationResult { distance, path })
}

/// Generates a N*N matrix half-filled with the distances in kilometers
/// between all following points.
///
/// - N: Number of points
///
/// ```text
///  +-----------------------> column/j
///  | - Y Y Y Y Y Y Y Y Y Y
///  | - - Y Y Y Y Y Y Y Y Y
///  | - - - Y Y Y Y Y Y Y Y
///  | - - - - Y Y Y Y Y Y Y
///  | - - - - - Y Y Y Y Y Y
///  | - - - - - - Y Y Y Y Y
///  | - - - - - - - Y Y Y Y
///  | - - - - - - - - Y Y Y
///  | - - - - - - - - - Y Y
///  | - - - - - - - - - - Y
///  | - - - - - - - - - - -
///  v
/// row/i
/// ```
///
fn following_dist_matrix(flat_points: &[FlatPoint<f32>]) -> Vec<Vec<f32>> {
    opt_par_iter(flat_points)
        .enumerate()
        .map(|(i, p1)| flat_points
            .iter()
            .skip(i + 1)
            .map(|p2| p1.distance(p2))
            .collect())
        .collect()
}

struct Graph {
    g: Vec<Vec<(usize, f32)>>,
}

impl Graph {
    fn from_distance_matrix(following_dist_matrix: &[Vec<f32>]) -> Self {
        let mut graph: Vec<Vec<(usize, f32)>> = Vec::with_capacity(LEGS);

        // leg: 5 -> 0
        for leg in (0..LEGS).rev() {
            debug!("-- Analyzing leg #{}", leg + 1);

            let layer = {
                let last_leg_dists = if leg == LEGS - 1 { None } else { Some(&graph[LEGS - 2 - leg]) };

                opt_into_par_iter(following_dist_matrix)
                    .enumerate()
                    .map(|(i, xxxdists)| xxxdists
                        .iter()
                        .enumerate()
                        .map(|(j, leg_dist)| {
                            let idx = j + i;
                            let last_leg_dist = last_leg_dists.map_or(0., |last| last[idx].1);
                            let total_dist = last_leg_dist + leg_dist;
                            (idx, total_dist)
                        })
                        .max_by_key(|&(_, dist)| OrdVar::new_checked(dist))
                        .unwrap_or((0, 0.)))
                    .collect()
            };

            graph.push(layer)
        }

        Graph { g: graph }
    }

    /// Finds the path through the `leg_distance_matrix` with the largest distance
    /// and returns an array with the corresponding `points` indices
    ///
    fn find_max_distance_path(&self) -> Path {
        let max_distance_index = self.g[LEGS - 1]
            .iter()
            .enumerate()
            .max_by_key(|&(_, (_, dist))| OrdVar::new_checked(dist))
            .unwrap()
            .0;

        let iter = GraphIterator {
            graph: self,
            next: Some((self.g.len(), max_distance_index))
        };

        let path = iter.collect::<Vec<_>>();

        assert_eq!(path.len(), LEGS + 1);

        path
    }
}

struct GraphIterator<'a> {
    graph: &'a Graph,
    next: Option<(usize, usize)>,
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
            let next_index = self.graph.g[next_layer][index].0;
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

/// Calculates the altitude difference between the start and the end point
/// of the given `path`.
fn find_altitude_delta<T: Point>(path: &Path, points: &[T]) -> i16 {
    let start_point = &points[*path.first().unwrap()];
    let end_point = &points[*path.last().unwrap()];
    start_point.altitude() - end_point.altitude()
}
