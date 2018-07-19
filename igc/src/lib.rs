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

    let seconds_since_midnight = caps[1].parse::<u32>().unwrap() * 3600 +
        caps[2].parse::<u32>().unwrap() * 60 +
        caps[3].parse::<u32>().unwrap();

    let latitude = caps[4].parse::<f64>().unwrap() +
        caps[5].parse::<f64>().unwrap() / 60000.;

    let longitude = caps[7].parse::<f64>().unwrap() +
        caps[8].parse::<f64>().unwrap() / 60000.;

    Fix {
        seconds_since_midnight,
        latitude: if &caps[6] == "S" { -latitude } else { latitude },
        longitude: if &caps[9] == "W" { -longitude } else { longitude },
        altitude_gps: caps[11].parse::<i16>().unwrap(),
        altitude_pressure: caps[12].parse::<i16>().unwrap(),
    }
}
