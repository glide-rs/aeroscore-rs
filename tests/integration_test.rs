#[macro_use]
extern crate assert_approx_eq;

extern crate aeroscore;
extern crate igc;

use aeroscore::olc;
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
    run_test(include_str!("fixtures/2017-08-14-fla-6ng-01.igc"), release, 501.3);
}

#[test]
fn distance_for_87i_qqk() {
    let release = Time::from_hms(09, 02, 05);
    run_test(include_str!("fixtures/87ilqqk1.igc"), release, 779.3);
}

#[test]
fn distance_for_99b_7r9() {
    let release = Time::from_hms(16, 54, 06);
    run_test(include_str!("fixtures/99bv7r92.igc"), release, 115.86);
}

fn run_test(file: &str, release: Time, expected_distance: f32) {
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

    let result = olc::optimize(&fixes).unwrap();

    assert_approx_eq!(result.distance, expected_distance, 0.1);
}
