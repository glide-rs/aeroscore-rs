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
}

#[allow(dead_code)]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return help();
    }

    &args[1..].iter().for_each(|path| analyze(path));
}

fn analyze(path: &str) {
    println!("--- {}", path);

    let current_dir = env::current_dir().expect("failed to determine current folder");
    let path = current_dir.join(path);

    let file = File::open(&path).expect("failed to open file");

    let fixes = BufReader::new(file)
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| l.starts_with('B'))
        .map(|line| Point(igc::parse_fix(&line)))
        .collect::<Vec<_>>();

    let result = olc::optimize(&fixes);

    println!("distance: {:.2} km", result.distance);
}

fn help() {
    println!("usage: aeroscore <igc-file>");
}
