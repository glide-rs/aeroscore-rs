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
}

#[test]
fn it_works() {
    let fixes = include_str!("fixtures/2017-08-14-fla-6ng-01.igc")
        .lines()
        .filter(|l| l.starts_with('B'))
        .map(|line| Point(igc::parse_fix(&line)))
        .collect::<Vec<_>>();

    let result = olc::optimize(&fixes);

    assert_approx_eq!(result.distance, 504.023, 0.01);
}
