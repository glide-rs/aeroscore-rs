#[macro_use]
extern crate assert_approx_eq;

extern crate aeroscore;
extern crate igc;

use aeroscore::olc;
use aeroscore::olc::OptimizationResult;
use igc::util::Time;

struct Point {
    latitude: f32,
    longitude: f32,
    altitude: i16,
}

impl aeroscore::Point for Point {
    fn latitude(&self) -> f32 {
        self.latitude
    }
    fn longitude(&self) -> f32 {
        self.longitude
    }
    fn altitude(&self) -> i16 {
        self.altitude
    }
}

#[test]
fn distance_for_78e_6ng() {
    let release = Time::from_hms(10, 28, 05);
    let result = run_test(include_str!("fixtures/2017-08-14-fla-6ng-01.igc"), release);
    assert_approx_eq!(result.distance, 501.3, 0.1);
}

#[test]
fn distance_for_87i_qqk() {
    let release = Time::from_hms(09, 02, 05);
    let result = run_test(include_str!("fixtures/87ilqqk1.igc"), release);
    assert_approx_eq!(result.distance, 782.74, 0.1);
}

#[test]
fn distance_for_99b_7r9() {
    let release = Time::from_hms(16, 54, 06);
    let result = run_test(include_str!("fixtures/99bv7r92.igc"), release);
    assert_approx_eq!(result.distance, 197.14, 0.1);
}

fn run_test(file: &str, release: Time) -> OptimizationResult {
    env_logger::try_init().ok();

    let fixes = file.lines()
        .filter(|l| l.starts_with('B'))
        .filter_map(|line| igc::records::BRecord::parse(&line).ok()
            .map_or(None, |record| {
                if record.timestamp.seconds_since_midnight() >= release.seconds_since_midnight() {
                    Some(Point {
                        latitude: record.pos.lat.into(),
                        longitude: record.pos.lon.into(),
                        altitude: record.gps_alt,
                    })
                } else {
                    None
                }
            }))
        .collect::<Vec<_>>();

    olc::optimize(&fixes).unwrap()
}
