#[macro_use]
extern crate assert_approx_eq;

extern crate aeroscore;
extern crate igc;

use aeroscore::olc;

struct Point {
    latitude: f64,
    longitude: f64,
    altitude: i16,
}

impl aeroscore::Point for Point {
    fn latitude(&self) -> f64 {
        self.latitude
    }
    fn longitude(&self) -> f64 {
        self.longitude
    }
    fn altitude(&self) -> i16 {
        self.altitude
    }
}

#[test]
fn it_works() {
    let release_seconds = 10 * 3600 + 28 * 60 + 5;

    let fixes = include_str!("fixtures/2017-08-14-fla-6ng-01.igc")
        .lines()
        .filter(|l| l.starts_with('B'))
        .filter_map(|line| igc::records::BRecord::parse(&line).ok()
            .map_or(None, |record| {
                if seconds_since_midnight(&record.timestamp) >= release_seconds {
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

    assert_approx_eq!(result.distance, 501.3, 0.1);
}

fn seconds_since_midnight(time: &igc::util::Time) -> i32 {
    time.hours as i32 * 60 * 60 + time.minutes as i32 * 60 + time.seconds as i32
}
