use flat_projection::{FlatProjection, FlatPoint};
use ord_subset::OrdSubsetIterExt;

use crate::Point;
use crate::parallel::*;

/// Projects all geographic points onto a flat surface for faster geodesic calculation
///
pub fn to_flat_points<T: Point>(points: &[T]) -> Vec<FlatPoint<f32>> {
    let center_lat = points.center_lat().unwrap();
    let center_lon = points.center_lon().unwrap();
    let proj = FlatProjection::new(center_lon, center_lat);

    opt_par_iter(points)
        .map(|fix| proj.project(fix.longitude(), fix.latitude()))
        .collect()
}

trait CenterLatitude {
    fn center_lat(self: &Self) -> Option<f32>;
}

impl<T: Point> CenterLatitude for [T] {
    fn center_lat(self: &Self) -> Option<f32> {
        let lat_min = self.iter().map(|fix| fix.latitude()).ord_subset_min()?;
        let lat_max = self.iter().map(|fix| fix.latitude()).ord_subset_max()?;

        Some((lat_min + lat_max) / 2.)
    }
}

trait CenterLongitude {
    fn center_lon(self: &Self) -> Option<f32>;
}

impl<T: Point> CenterLongitude for [T] {
    fn center_lon(self: &Self) -> Option<f32> {
        let lon_min = self.iter().map(|fix| fix.longitude()).ord_subset_min()?;
        let lon_max = self.iter().map(|fix| fix.longitude()).ord_subset_max()?;

        Some((lon_min + lon_max) / 2.)
    }
}
