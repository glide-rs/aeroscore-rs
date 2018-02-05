use failure::Error;
use flat_projection::{FlatProjection, FlatPoint};
use rayon::prelude::*;

const LEGS: usize = 6;

pub trait Point: Sync {
    fn latitude(&self) -> f64;
    fn longitude(&self) -> f64;
}

#[derive(Debug)]
pub struct OptimizationResult {
    pub point_list: Vec<usize>,
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

fn to_flat_points<T: Point>(points: &[T]) -> Vec<FlatPoint<f64>> {
    let center = center_lat(points);
    let proj = FlatProjection::new(center);

    points.par_iter()
        .map(|fix| proj.project(fix.longitude(), fix.latitude()))
        .collect()
}

fn center_lat<T: Point>(fixes: &[T]) -> f64 {
    let lat_min = fixes.iter().map(|fix| fix.latitude()).min_by(|a, b| a.partial_cmp(&b).expect("lat_min min_by")).expect("lat_min failed");
    let lat_max = fixes.iter().map(|fix| fix.latitude()).max_by(|a, b| a.partial_cmp(&b).expect("lon_min min_by")).expect("lat_max failed");

    (lat_min + lat_max) / 2.
}

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
                    .max_by(|&(_, dist1), &(_, dist2)| dist1.partial_cmp(&dist2).unwrap())
                    .unwrap_or((0, 0.)))
                .collect()
        };

        dists.push(leg_dists)
    }

    return dists;
}

fn find_max_distance_path(leg_distance_matrix: &[Vec<(usize, f64)>]) -> Vec<usize> {
    let mut point_list: Vec<usize> = vec![0; LEGS + 1];

    point_list[LEGS] = leg_distance_matrix[LEGS - 1]
        .iter()
        .enumerate()
        .max_by(|&(_, dist1), &(_, dist2)| dist1.partial_cmp(&dist2).unwrap())
        .map_or(0, |it| it.0);

    // find waypoints
    for leg in (0..LEGS).rev() {
        point_list[leg] = leg_distance_matrix[leg][point_list[leg + 1]].0;
    }

    return point_list;
}

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
