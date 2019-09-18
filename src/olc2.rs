use failure::Error;
use flat_projection::{FlatProjection, FlatPoint};
use ord_subset::OrdSubsetIterExt;

cfg_if! {
    if #[cfg(feature = "rayon")] {
        use rayon::slice;
        use rayon::prelude::*;

        fn opt_par_iter<T: Sync>(x: &[T]) -> slice::Iter<T> {
            x.par_iter()
        }

        fn opt_into_par_iter<T: Sync>(x: &[T]) -> slice::Iter<T> {
            x.into_par_iter()
        }

    } else {
        use std::slice;

        fn opt_par_iter<T>(x: &[T]) -> slice::Iter<T> {
            x.iter()
        }

        fn opt_into_par_iter<T>(x: &[T]) -> slice::Iter<T> {
            x.into_iter()
        }
    }
}

const LEGS: usize = 6;
const POINTS: usize = LEGS + 1;

pub trait Point: Sync + std::fmt::Debug {
    fn latitude(&self) -> f64;
    fn longitude(&self) -> f64;
    fn altitude(&self) -> i16;
}

#[derive(Debug)]
pub struct OptimizationResult {
    pub point_list: [usize; POINTS],
    pub distance: f64,
}

pub fn optimize<T: Point>(route: &[T]) -> Result<OptimizationResult, Error> {
//    let flat_points = to_flat_points(route);
//    let distance_matrix = calculate_distance_matrix(&flat_points);
    let point_list = opt(route);
//    let point_list = opt(&distance_matrix);
//    let point_list = find_max_distance_path(&middle_leg_distance_matrix, route);
    let distance = calculate_distance(route, &point_list);

    Ok(OptimizationResult { distance, point_list })
}

/// Projects all geographic points onto a flat surface for faster geodesic calculation
///
fn to_flat_points<T: Point>(points: &[T]) -> Vec<FlatPoint<f64>> {
    let center = points.center_lat().unwrap();
    let proj = FlatProjection::new(center);

    opt_par_iter(points)
        .map(|fix| proj.project(fix.longitude(), fix.latitude()))
        .collect()
}

trait CenterLatitude {
    fn center_lat(self: &Self) -> Option<f64>;
}

impl<T: Point> CenterLatitude for [T] {
    fn center_lat(self: &Self) -> Option<f64> {
        let lat_min = self.iter().map(|fix| fix.latitude()).ord_subset_min()?;
        let lat_max = self.iter().map(|fix| fix.latitude()).ord_subset_max()?;

        Some((lat_min + lat_max) / 2.)
    }
}

/// Generates a N*N matrix half-filled with the distances in kilometers between all points.
///
/// - N: Number of points
///
/// ```text
///  +-----------------------> Y
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
///  X
/// ```
///
fn calculate_distance_matrix(flat_points: &[FlatPoint<f64>]) -> Vec<Vec<f64>> {
    opt_par_iter(flat_points)
        .enumerate()
        .map(|(i, p1)| flat_points
            .iter()
            .take(i)
            .map(|p2| p1.distance(p2))
            .collect())
        .collect()
}

/// potential C points, with best B and total distance
/// potential D points, with best C and total distance
/// potential E points, with best D and total distance
/// potential F points, with best E and total distance
fn opt<T: Point>(route: &[T]) -> [usize; LEGS + 1] {
//    let distance_matrix = calculate_distance_matrix(&flat_points);

    let flat_points = to_flat_points(route);

    // BC distances for each potential point C
    let c_candidates = flat_points
        .iter()
        .enumerate()
        .map(|(c_index, c_point)| flat_points
            .iter()
            .take(c_index + 1)
            .enumerate()
            .map(|(b_index, b_point)| (b_index, b_point.distance(c_point)))
            .ord_subset_max_by_key(|&(_, dist)| dist)
            .unwrap())
        .collect::<Vec<_>>();

    // CD distances for each potential point D
    let d_candidates = flat_points
        .iter()
        .enumerate()
        .map(|(d_index, d_point)| flat_points
            .iter()
            .take(d_index + 1)
            .enumerate()
            .map(|(c_index, c_point)| {
                let (_b_index, bc_distance) = c_candidates[c_index];
                let cd_distance = c_point.distance(d_point);
                (c_index, bc_distance + cd_distance)
            })
            .ord_subset_max_by_key(|&(_, dist)| dist)
            .unwrap())
        .collect::<Vec<_>>();

    // DE distances for each potential point E
    let e_candidates = flat_points
        .iter()
        .enumerate()
        .map(|(e_index, e_point)| flat_points
            .iter()
            .take(e_index + 1)
            .enumerate()
            .map(|(d_index, d_point)| {
                let (_c_index, bd_distance) = d_candidates[d_index];
                let de_distance = d_point.distance(e_point);
                (d_index, bd_distance + de_distance)
            })
            .ord_subset_max_by_key(|&(_, dist)| dist)
            .unwrap())
        .collect::<Vec<_>>();

    // EF distances for each potential point F
    let f_candidates = flat_points
        .iter()
        .enumerate()
        .map(|(f_index, f_point)| flat_points
            .iter()
            .take(f_index + 1)
            .enumerate()
            .map(|(e_index, e_point)| {
                let (_d_index, be_distance) = e_candidates[e_index];
                let ef_distance = e_point.distance(f_point);
                (e_index, be_distance + ef_distance)
            })
            .ord_subset_max_by_key(|&(_, dist)| dist)
            .unwrap())
        .collect::<Vec<_>>();

    let g_candidates = flat_points
        .iter()
        .enumerate()
        .map(|(g_index, g_point)| {
            let g_geo_point = &route[g_index];

            flat_points
            .iter()
            .take(g_index + 1)
            .enumerate()
            .map(|(f_index, f_point)| {
                let fg_distance = f_point.distance(g_point);

                let (e_index, bf_distance) = f_candidates[f_index];
                let (d_index, _) = e_candidates[e_index];
                let (c_index, _) = d_candidates[d_index];
                let (b_index, _) = c_candidates[c_index];
                let b_point = flat_points[b_index];

                flat_points
                    .iter()
                    .take(b_index + 1)
                    .enumerate()
                    .filter(|&(a_index, _a_point)| {
                        let a_geo_point = &route[a_index];
                        a_geo_point.altitude() - g_geo_point.altitude() <= 1000
                    })
                    .map(|(a_index, a_point)| {
                        let ab_distance = a_point.distance(&b_point);
                        ((a_index, b_index, c_index, d_index, e_index, f_index, g_index), ab_distance + bf_distance + fg_distance)
                    })
                    .ord_subset_max_by_key(|&(_, dist)| dist)
                    .unwrap()
            })
            .ord_subset_max_by_key(|&(_, dist)| dist)
            .unwrap()})
        .ord_subset_max_by_key(|&(_, dist)| dist)
        .unwrap();

    println!("{:#?}", g_candidates);

    let ((a_index, b_index, c_index, d_index, e_index, f_index, g_index), _distance) = g_candidates;

    let a_geo_point = &route[a_index];
    let b_geo_point = &route[b_index];
    let c_geo_point = &route[c_index];
    let d_geo_point = &route[d_index];
    let e_geo_point = &route[e_index];
    let f_geo_point = &route[f_index];
    let g_geo_point = &route[g_index];

    println!("{:?}", a_geo_point);
    println!("{:?}", b_geo_point);
    println!("{:?}", c_geo_point);
    println!("{:?}", d_geo_point);
    println!("{:?}", e_geo_point);
    println!("{:?}", f_geo_point);
    println!("{:?}", g_geo_point);

    let mut point_list: [usize; LEGS + 1] = [a_index, b_index, c_index, d_index, e_index, f_index, g_index];
    point_list
}

/// Finds the path through the `leg_distance_matrix` with the largest distance
/// and returns an array with the corresponding `points` indices
///
fn find_max_distance_path<T: Point>(leg_distance_matrix: &[Vec<(usize, f64)>], points: &[T]) -> [usize; LEGS + 1] {
    let max_distance_finish_index = leg_distance_matrix[LEGS - 1]
        .iter()
        .enumerate()
//        .filter(|&(finish_index, _)| {
//            let path = find_path(leg_distance_matrix, finish_index);
//            let start_index = path[0];
//            let start = &points[start_index];
//            let finish = &points[finish_index];
//            finish.altitude() + 1000 >= start.altitude()
//        })
        .ord_subset_max_by_key(|&(_, dist)| dist)
        .map_or(0, |it| it.0);

    find_path(leg_distance_matrix, max_distance_finish_index)
}

fn find_path(leg_distance_matrix: &[Vec<(usize, f64)>], finish_index: usize) -> [usize; LEGS + 1] {
    let mut point_list: [usize; LEGS + 1] = [0; LEGS + 1];

    point_list[LEGS] = finish_index;

    // find waypoints
    for leg in (0..LEGS).rev() {
        point_list[leg] = leg_distance_matrix[leg][point_list[leg + 1]].0;
    }

    point_list
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
