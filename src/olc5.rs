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

fn opt<T: Point>(geo_points: &[T]) -> [usize; POINTS] {
    let num_points = geo_points.len();
    println!("num: {}", num_points);

//    let flat_points = to_flat_points(geo_points);
    // let distance_matrix = calculate_distance_matrix(&flat_points);
    let distances = gen_distances(&geo_points);

//    const N: usize = num; // Number of points
    const K: usize = 7; // Maximum number of nodes allowed on the path
    let (distance, indices) = fast_and_maybe_wrong(num_points, K, &geo_points, &distances);

    let a_index = indices[0];
    let b_index = indices[1];
    let c_index = indices[2];
    let d_index = indices[3];
    let e_index = indices[4];
    let f_index = indices[5];
    let g_index = indices[6];

    let a_geo_point = &geo_points[a_index];
    let b_geo_point = &geo_points[b_index];
    let c_geo_point = &geo_points[c_index];
    let d_geo_point = &geo_points[d_index];
    let e_geo_point = &geo_points[e_index];
    let f_geo_point = &geo_points[f_index];
    let g_geo_point = &geo_points[g_index];

    println!("{:?}", a_geo_point);
    println!("{:?}", b_geo_point);
    println!("{:?}", c_geo_point);
    println!("{:?}", d_geo_point);
    println!("{:?}", e_geo_point);
    println!("{:?}", f_geo_point);
    println!("{:?}", g_geo_point);

    let mut point_list: [usize; POINTS] = [a_index, b_index, c_index, d_index, e_index, f_index, g_index];
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

fn fast_and_maybe_wrong<T: Point>(N: usize, K: usize, points: &[T], distances: &Vec<Vec<f64>>) -> (f64, Vec<usize>) {
    // dp[k][i] is a tuple containing information about the longest path using at most `k` nodes and ending at point `i`.
    // dp[k][i].0 is the total length of the path.
    // dp[k][i].1 is the index of the previous point before point `i` (i.e. the predecessor) along the path.
    // This variable is often called "dp" for "dynamic programming", *shrug*.
    let mut dp = vec![vec![(0.0, 0); N]; K + 1];

    for k in 1..=K {
        for j in 1..N { // the point we're going to
            for i in 0..j { // the point we're coming from
                if k == K && points[i].altitude() > points[j].altitude() + 1000 {
                    continue;
                }

                let total_length = dp[k - 1][i].0 + distances[i][j];
                if dp[k][j].0 < total_length {
                    dp[k][j] = (total_length, i);
                }
            }
        }
    }

    // Find the dp data for the longest path using at most `K` segments
    let (last_point_index, _) = dp[K]
        .iter()
        .enumerate()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .expect("no solution");

    // Read the total length.
    let total_length = dp[K][last_point_index].0;

    // Read the whole path out from the rest of the dp array.
    let mut path = vec![last_point_index];

    for k in (1..K).rev() {
        let i = *path.last().unwrap();
        path.push(dp[k][i].1);

        // We've reached the beginning of the path
        if dp[k][i].0 == 0.0 {
            break;
        }
    }

    path.reverse();

    return (total_length, path);
}

fn gen_distances<T: Point>(points: &[T]) -> Vec<Vec<f64>> {
    let n = points.len();
    let mut distances = vec![vec![0.0; n]; n];

    for j in 1..n {
        for i in 0..j {
            let dist = haversine_distance(&points[i], &points[j]);
            distances[i][j] = dist;
        }
    }

    distances
}
