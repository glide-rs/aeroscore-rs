use crate::Point;

pub fn haversine_distance(fix1: &dyn Point, fix2: &dyn Point) -> f64 {
    const R: f64 = 6371.; // kilometres

    let phi1 = fix1.latitude().to_radians();
    let phi2 = fix2.latitude().to_radians();
    let delta_phi = (fix2.latitude() - fix1.latitude()).to_radians();
    let delta_rho = (fix2.longitude() - fix1.longitude()).to_radians();
    let delta_phi_half_sin = (delta_phi / 2.).sin();
    let delta_rho_half_sin = (delta_rho / 2.).sin();

    let a = delta_phi_half_sin * delta_phi_half_sin +
        phi1.cos() * phi2.cos() * delta_rho_half_sin * delta_rho_half_sin;

    let c = 2. * a.sqrt().atan2((1. - a).sqrt());

    R * c
}
