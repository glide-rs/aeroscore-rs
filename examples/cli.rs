extern crate aeroscore;
extern crate igc;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use aeroscore::olc;

struct Point {
    time: igc::util::Time,
    latitude: f32,
    longitude: f32,
    altitude: i16,
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lon_min = (self.longitude - self.longitude.floor()) * 60.;
        let lat_min = (self.latitude - self.latitude.floor()) * 60.;
        write!(
            f,
            "{:02}:{:02}:{:02}  {:03.0}°{:06.3}E  {:03.0}°{:06.3}N  {:5.0}m",
            self.time.hours,
            self.time.minutes,
            self.time.seconds,
            self.longitude.floor(),
            lon_min,
            self.latitude.floor(),
            lat_min,
            self.altitude
        )
    }
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

#[allow(dead_code)]
fn main() {
    env_logger::init();

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
                altitude: record.pressure_alt,
            }))
        .collect::<Vec<_>>();

    println!("num points: {}", fixes.len());
    println!();

    let result = olc::optimize(&fixes).unwrap();

    println!("{:5}:  {:?}", result.path[0], fixes[result.path[0]]);
    println!("{:5}:  {:?}", result.path[1], fixes[result.path[1]]);
    println!("{:5}:  {:?}", result.path[2], fixes[result.path[2]]);
    println!("{:5}:  {:?}", result.path[3], fixes[result.path[3]]);
    println!("{:5}:  {:?}", result.path[4], fixes[result.path[4]]);
    println!("{:5}:  {:?}", result.path[5], fixes[result.path[5]]);
    println!("{:5}:  {:?}", result.path[6], fixes[result.path[6]]);
    println!();
    println!("distance: {:.2} km", result.distance);
}

fn help() {
    println!("usage: aeroscore <igc-file>");
}
