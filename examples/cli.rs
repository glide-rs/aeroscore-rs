extern crate aeroscore;
extern crate igc;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use aeroscore::olc;

struct Point {
    time: igc::util::Time,
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

    let fixes = BufReader::new(file)
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| l.starts_with('B'))
        .filter_map(|line| igc::records::BRecord::parse(&line).ok()
            .map(|record| Point {
                time: record.timestamp,
                latitude: record.pos.lat.into(),
                longitude: record.pos.lon.into(),
                altitude: record.gps_alt,
            }))
        .collect::<Vec<_>>();

    let result = olc::optimize(&fixes).unwrap();

    println!("distance: {:.2} km", result.distance);
}

fn help() {
    println!("usage: aeroscore <igc-file>");
}
