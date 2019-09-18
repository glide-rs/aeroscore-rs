extern crate aeroscore;
extern crate igc;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

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

#[allow(dead_code)]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return help();
    }

    args[1..].iter().for_each(|path| analyze(path));
}

fn analyze(path: &str) {
    println!("--- {}", path);

    let file = File::open(&path).expect("failed to open file");

    // 09:02:05
    let release_seconds = 9 * 3600 + 2 * 60 + 5;

    let fixes = BufReader::new(file)
        .lines()
        .filter_map(|l| l.ok())
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

    println!("distance: {:.2} km", result.distance);

    let start_fix = &fixes[result.point_list[0]];
    let finish_fix = &fixes[result.point_list[6]];

    let delta_alt = start_fix.altitude - finish_fix.altitude;
    if delta_alt > 1000 {
        println!("ERROR Startüberhöhung!!! {}m", delta_alt);
    }
}

fn seconds_since_midnight(time: &igc::util::Time) -> i32 {
    time.hours as i32 * 60 * 60 + time.minutes as i32 * 60 + time.seconds as i32
}

fn help() {
    println!("usage: aeroscore <igc-file>");
}
