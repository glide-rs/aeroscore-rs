#[macro_use]
extern crate assert_approx_eq;

extern crate aeroscore;
extern crate igc;

use aeroscore::olc3 as olc;

struct Point {
    time: igc::util::Time,
    latitude: f64,
    longitude: f64,
    altitude: i16,
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lon_min = (self.longitude - self.longitude.floor()) * 60.;
        let lat_min = (self.latitude - self.latitude.floor()) * 60.;
        write!(f, "{}  {:03.0}°{:06.3}E  {:03.0}°{:06.3}N  {:4.0}m", self.time, self.longitude.floor(), lon_min, self.latitude.floor(), lat_min, self.altitude)
    }
}

impl olc::Point for Point {
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
    // 09:02:05
    let release_seconds = 9 * 3600 + 2 * 60 + 5;

    let fixes = include_str!("fixtures/87ilqqk1.igc")
        .lines()
        .filter(|l| l.starts_with('B'))
        .filter_map(|line| igc::records::BRecord::parse(&line).ok()
            .map_or(None, |record| {
                if seconds_since_midnight(&record.timestamp) >= release_seconds {
                    Some(Point {
                        time: record.timestamp,
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

    assert_approx_eq!(result.distance, 780.4, 0.1);
}

fn seconds_since_midnight(time: &igc::util::Time) -> i32 {
    time.hours as i32 * 60 * 60 + time.minutes as i32 * 60 + time.seconds as i32
}
