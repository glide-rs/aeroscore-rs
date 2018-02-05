#[macro_use]
extern crate criterion;
#[macro_use]
extern crate lazy_static;

extern crate regex;
extern crate aeroscore;

use criterion::Criterion;
use regex::Regex;
use aeroscore::olc;

#[derive(Debug, Clone, Copy)]
struct Fix {
    latitude: f64,
    longitude: f64,
    altitude_gps: i16,
    altitude_pressure: i16,
}

impl olc::Point for Fix {
    fn latitude(&self) -> f64 {
        self.latitude
    }

    fn longitude(&self) -> f64 {
        self.longitude
    }
}

fn parse_fix(line: &str) -> Fix {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?x)
            ^B                     # record typ
            (\d{2})(\d{2})(\d{2})  # UTC time
            (\d{2})(\d{5})([NS])   # latitude
            (\d{3})(\d{5})([EW])   # longitude
            ([A-Z])                # validity
            (\d{5}|-\d{4})         # gps altitude
            (\d{5}|-\d{4})         # pressure altitude
        ").unwrap();
    }

    let caps = RE.captures(line).expect("Broken B record");

    let latitude = caps.at(4).unwrap().parse::<f64>().unwrap() +
        caps.at(5).unwrap().parse::<f64>().unwrap() / 60000.;

    let longitude = caps.at(7).unwrap().parse::<f64>().unwrap() +
        caps.at(8).unwrap().parse::<f64>().unwrap() / 60000.;

    Fix {
        latitude: if caps.at(6).unwrap() == "S" { -latitude } else { latitude },
        longitude: if caps.at(9).unwrap() == "W" { -longitude } else { longitude },
        altitude_gps: caps.at(11).unwrap().parse::<i16>().unwrap(),
        altitude_pressure: caps.at(12).unwrap().parse::<i16>().unwrap(),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("olc_classic", |b| b.iter(|| {
        let fixes = include_str!("../tests/fixtures/2017-08-14-fla-6ng-01.igc")
            .lines()
            .filter(|l| l.starts_with('B'))
            .map(|line| parse_fix(&line))
            .collect::<Vec<_>>();

        olc::optimize(&fixes)
    }));
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10);

    targets = criterion_benchmark
}
criterion_main!(benches);
