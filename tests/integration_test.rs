#[macro_use]
extern crate assert_approx_eq;

extern crate aeroscore;
extern crate igc;

use aeroscore::olc;

struct Point(igc::Fix);

impl olc::Point for Point {
    fn latitude(&self) -> f64 {
        self.0.latitude
    }
    fn longitude(&self) -> f64 {
        self.0.longitude
    }
    fn altitude(&self) -> i16 {
        self.0.altitude_gps
    }
}

#[test]
fn it_works() {
    let release_seconds = 10 * 3600 + 28 * 60 + 05;

    let fixes = include_str!("fixtures/2017-08-14-fla-6ng-01.igc")
        .lines()
        .filter(|l| l.starts_with('B'))
        .map(|line| Point(igc::parse_fix(&line)))
        .filter(|point| point.0.seconds_since_midnight >= release_seconds)
        .collect::<Vec<_>>();

    let result = olc::optimize(&fixes).unwrap();

    assert_approx_eq!(result.distance, 501.3, 0.1);
}
