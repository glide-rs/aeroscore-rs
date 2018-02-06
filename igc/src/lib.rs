#[macro_use]
extern crate lazy_static;

extern crate regex;

use regex::Regex;

#[derive(Debug, Clone, Copy)]
pub struct Fix {
    pub seconds_since_midnight: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_gps: i16,
    pub altitude_pressure: i16,
}

pub fn parse_fix(line: &str) -> Fix {
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

    let seconds_since_midnight = caps.at(1).unwrap().parse::<u32>().unwrap() * 3600 +
        caps.at(2).unwrap().parse::<u32>().unwrap() * 60 +
        caps.at(3).unwrap().parse::<u32>().unwrap();

    let latitude = caps.at(4).unwrap().parse::<f64>().unwrap() +
        caps.at(5).unwrap().parse::<f64>().unwrap() / 60000.;

    let longitude = caps.at(7).unwrap().parse::<f64>().unwrap() +
        caps.at(8).unwrap().parse::<f64>().unwrap() / 60000.;

    Fix {
        seconds_since_midnight,
        latitude: if caps.at(6).unwrap() == "S" { -latitude } else { latitude },
        longitude: if caps.at(9).unwrap() == "W" { -longitude } else { longitude },
        altitude_gps: caps.at(11).unwrap().parse::<i16>().unwrap(),
        altitude_pressure: caps.at(12).unwrap().parse::<i16>().unwrap(),
    }
}
