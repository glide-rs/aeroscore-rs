#[macro_use] extern crate lazy_static;
#[macro_use] extern crate assert_approx_eq;

extern crate regex;
extern crate aeroscore;

use regex::Regex;
use aeroscore::olc;

fn parse_fix(line: &str) -> olc::Fix {
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

    olc::Fix {
        latitude: if caps.at(6).unwrap() == "S" { -latitude } else { latitude },
        longitude: if caps.at(9).unwrap() == "W" { -longitude } else { longitude },
        altitude_gps: caps.at(11).unwrap().parse::<i16>().unwrap(),
        altitude_pressure: caps.at(12).unwrap().parse::<i16>().unwrap()
    }
}

#[test]
fn it_works() {
    let fixes = include_str!("fixtures/2017-08-14-fla-6ng-01.igc")
        .lines()
        .filter(|l| l.starts_with('B'))
        .map(|line| parse_fix(&line))
        .collect::<Vec<_>>();

    let result = olc::optimize(&fixes);

    assert_approx_eq!(result.distance, 504.023, 0.01);
}
