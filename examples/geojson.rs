#[macro_use]
extern crate serde_json;

extern crate aeroscore;
extern crate igc;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

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

#[allow(dead_code)]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return help();
    }

    analyze(&args[1]);
}

fn analyze(path: &str) {
    let file = File::open(&path).expect("failed to open file");

    let fixes = BufReader::new(file)
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| l.starts_with('B'))
        .filter_map(|line| igc::records::BRecord::parse(&line).ok()
            .map(|record| Point {
                latitude: record.pos.lat.into(),
                longitude: record.pos.lon.into(),
                altitude: record.gps_alt,
            }))
        .collect::<Vec<_>>();

    let result = olc::optimize(&fixes).unwrap();

    let result_coords = result.point_list.iter()
        .map(|i| &fixes[*i])
        .map(|p| (p.longitude, p.latitude))
        .collect::<Vec<_>>();

    let json = json!({
      "type": "FeatureCollection",
      "features": [
        {
          "id": "gps-track",
          "type": "Feature",
          "properties": {
            "stroke": "#005717",
          },
          "geometry": {
            "type": "LineString",
            "coordinates": fixes.iter().map(|p| (p.longitude, p.latitude)).collect::<Vec<_>>(),
          },
        },
        {
          "id": "olc",
          "type": "Feature",
          "properties": {
            "distance": result.distance,
            "stroke": "#ff40ff",
          },
          "geometry": {
            "type": "LineString",
            "coordinates": result_coords,
          },
        },
      ],
    });

    println!("{}", json.to_string());
}

fn help() {
    println!("usage: aeroscore <igc-file>");
}
