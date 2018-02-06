use failure::Error;
use flat_projection::{FlatProjection, FlatPoint};
use ord_subset::OrdSubsetIterExt;
use rayon::prelude::*;

const LEGS: usize = 6;

pub trait Point: Sync {
    fn latitude(&self) -> f64;
    fn longitude(&self) -> f64;
    fn altitude(&self) -> i16;
}

#[derive(Debug)]
pub struct OptimizationResult {
    pub point_list: [usize; LEGS + 1],
    pub distance: f64,
}

pub fn optimize<T: Point>(route: &[T]) -> Result<OptimizationResult, Error> {
    let flat_points = to_flat_points(route);
    let distance_matrix = calculate_distance_matrix(&flat_points);
    let leg_distance_matrix = calculate_leg_distance_matrix(&distance_matrix);
    let point_list = find_max_distance_path(&leg_distance_matrix);
    let distance = calculate_distance(route, &point_list);

    Ok(OptimizationResult { distance, point_list })
}

/// Projects all geographic points onto a flat surface for faster geodesic calculation
///
fn to_flat_points<T: Point>(points: &[T]) -> Vec<FlatPoint<f64>> {
    let center = center_lat(points);
    let proj = FlatProjection::new(center);

    points.par_iter()
        .map(|fix| proj.project(fix.longitude(), fix.latitude()))
        .collect()
}

fn center_lat<T: Point>(fixes: &[T]) -> f64 {
    let lat_min = fixes.iter().map(|fix| fix.latitude()).ord_subset_min().expect("lat_min failed");
    let lat_max = fixes.iter().map(|fix| fix.latitude()).ord_subset_max().expect("lat_max failed");

    (lat_min + lat_max) / 2.
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
fn calculate_distance_matrix(flat_points: &[FlatPoint<f64>]) -> Vec<Vec<f64>> {
    flat_points.par_iter()
        .enumerate()
        .map(|(i, p1)| flat_points
            .iter()
            .take(i)
            .map(|p2| p1.distance(&p2))
            .collect())
        .collect()
}

fn calculate_leg_distance_matrix(distance_matrix: &[Vec<f64>]) -> Vec<Vec<(usize, f64)>> {
    let mut dists: Vec<Vec<(usize, f64)>> = Vec::with_capacity(LEGS);

    for leg in 0..LEGS {
        let leg_dists = {
            let last_leg_dists = if leg == 0 { None } else { Some(&dists[leg - 1]) };

            (&distance_matrix)
                .into_par_iter()
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

        dists.push(leg_dists)
    }

    return dists;
}

/// Finds the path through the `leg_distance_matrix` with the largest distance
/// and returns an array with the corresponding `points` indices
///
fn find_max_distance_path(leg_distance_matrix: &[Vec<(usize, f64)>]) -> [usize; LEGS + 1] {
    let max_distance_finish_index = leg_distance_matrix[LEGS - 1]
        .iter()
        .enumerate()
        .ord_subset_max_by_key(|&(_, dist)| dist)
        .map_or(0, |it| it.0);

    return find_path(leg_distance_matrix, max_distance_finish_index);
}

fn find_path(leg_distance_matrix: &[Vec<(usize, f64)>], finish_index: usize) -> [usize; LEGS + 1] {
    let mut point_list: [usize; LEGS + 1] = [0; LEGS + 1];

    point_list[LEGS] = finish_index;

    // find waypoints
    for leg in (0..LEGS).rev() {
        point_list[leg] = leg_distance_matrix[leg][point_list[leg + 1]].0;
    }

    return point_list;
}

/// Calculates the total task distance (via haversine algorithm) from
/// the original `route` and the arry of indices
///
fn calculate_distance<T: Point>(route: &[T], point_list: &[usize]) -> f64 {
    (0..LEGS)
        .map(|i| (point_list[i], point_list[i + 1]))
        .map(|(i1, i2)| (&route[i1], &route[i2]))
        .map(|(fix1, fix2)| haversine_distance(fix1, fix2))
        .sum()
}

fn haversine_distance(fix1: &Point, fix2: &Point) -> f64 {
    const R: f64 = 6371.; // kilometres

    let phi1 = fix1.latitude().to_radians();
    let phi2 = fix2.latitude().to_radians();
    let delta_phi = (fix2.latitude() - fix1.latitude()).to_radians();
    let delta_rho = (fix2.longitude() - fix1.longitude()).to_radians();

    let a = (delta_phi / 2.).sin() * (delta_phi / 2.).sin() +
        phi1.cos() * phi2.cos() *
            (delta_rho / 2.).sin() * (delta_rho / 2.).sin();

    let c = 2. * a.sqrt().atan2((1. - a).sqrt());

    R * c
}
