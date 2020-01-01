use failure::Error;
use flat_projection::FlatPoint;
use ord_subset::OrdSubsetIterExt;

use crate::Point;
use crate::flat::to_flat_points;
use crate::haversine::haversine_distance;
use crate::parallel::*;

const LEGS: usize = 6;

pub type Path = Vec<usize>;
pub type Graph = Vec<Vec<(usize, f32)>>;

#[derive(Debug)]
pub struct OptimizationResult {
    pub point_list: Path,
    pub distance: f32,
}

pub fn optimize<T: Point>(route: &[T]) -> Result<OptimizationResult, Error> {
    let flat_points = to_flat_points(route);
    let distance_matrix = calculate_distance_matrix(&flat_points);
    let graph = find_graph(&distance_matrix);
    let point_list = find_max_distance_path(&graph);
    let distance = calculate_distance(route, &point_list);

    Ok(OptimizationResult { distance, point_list })
}

/// Generates a N*N matrix half-filled with the distances in kilometers between all points.
///
/// - N: Number of points
///
/// ```text
///  +-----------------------> column
///  | - - - - - - - - - - -
///  | X - - - - - - - - - -
///  | X X - - - - - - - - -
///  | X X X - - - - - - - -
///  | X X X X - - - - - - -
///  | X X X X X - - - - - -
///  | X X X X X X - - - - -
///  | X X X X X X X - - - -
///  | X X X X X X X X - - -
///  | X X X X X X X X X - -
///  | X X X X X X X X X X -
///  v
/// row
/// ```
///
fn calculate_distance_matrix(flat_points: &[FlatPoint<f32>]) -> Vec<Vec<f32>> {
    opt_par_iter(flat_points)
        .enumerate()
        .map(|(i, p1)| flat_points
            .iter()
            .take(i)
            .map(|p2| p1.distance(p2))
            .collect())
        .collect()
}

fn find_graph(distance_matrix: &[Vec<f32>]) -> Graph {
    let mut graph: Graph = Vec::with_capacity(LEGS);

    for leg in 0..LEGS {
        let leg_dists = {
            let last_leg_dists = if leg == 0 { None } else { Some(&graph[leg - 1]) };

            opt_into_par_iter(distance_matrix)
                .map(|xxxdists| xxxdists
                    .iter()
                    .enumerate()
                    .map(|(j, leg_dist)| {
                        let last_leg_dist = last_leg_dists.map_or(0., |last| last[j].1);
                        let total_dist = last_leg_dist + leg_dist;
                        (j, total_dist)
                    })
                    .ord_subset_max_by_key(|&(_, dist)| dist)
                    .unwrap_or((0, 0.)))
                .collect()
        };

        graph.push(leg_dists)
    }

    graph
}

/// Finds the path through the `leg_distance_matrix` with the largest distance
/// and returns an array with the corresponding `points` indices
///
fn find_max_distance_path(graph: &Graph) -> Path {
    let mut path = vec![0; LEGS + 1];

    path[LEGS] = graph[LEGS - 1]
        .iter()
        .enumerate()
        .ord_subset_max_by_key(|&(_, (_, dist))| dist)
        .map_or(0, |it| it.0);

    // find waypoints
    for leg in (0..LEGS).rev() {
        path[leg] = graph[leg][path[leg + 1]].0;
    }

    assert_eq!(path.len(), LEGS + 1);

    path
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
