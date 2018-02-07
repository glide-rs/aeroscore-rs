#[macro_use]
extern crate serde_json;

extern crate aeroscore;
extern crate igc;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

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
        .map(|line| Point(igc::parse_fix(&line)))
        .collect::<Vec<_>>();

    let result = olc::optimize(&fixes).unwrap();

    let result_coords = result.point_list.iter()
        .map(|i| &fixes[*i])
        .map(|p| (p.0.longitude, p.0.latitude))
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
            "coordinates": fixes.iter().map(|p| (p.0.longitude, p.0.latitude)).collect::<Vec<_>>(),
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
