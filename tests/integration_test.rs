#[macro_use]
extern crate assert_approx_eq;

extern crate aeroscore;
extern crate igc;

use aeroscore::olc;

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
    let release_seconds = 10 * 3600 + 28 * 60 + 5;
    run_test(include_str!("fixtures/2017-08-14-fla-6ng-01.igc"), release_seconds, 501.3);
}

#[test]
fn distance_for_87i_qqk() {
    let release_seconds = 9 * 3600 + 2 * 60 + 5;
    run_test(include_str!("fixtures/87ilqqk1.igc"), release_seconds, 779.3);
}

fn run_test(file: &str, release_seconds: u32, expected_distance: f32) {
    let fixes = file.lines()
        .filter(|l| l.starts_with('B'))
        .filter_map(|line| igc::records::BRecord::parse(&line).ok()
            .map_or(None, |record| {
                if record.timestamp.seconds_since_midnight() >= release_seconds {
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
