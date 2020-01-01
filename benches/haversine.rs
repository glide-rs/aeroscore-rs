#[macro_use]
extern crate criterion;

extern crate aeroscore;
extern crate igc;

use criterion::Criterion;
use aeroscore::haversine::haversine_distance;

struct Point {
    latitude: f32,
    longitude: f32,
}

impl aeroscore::Point for Point {
    fn latitude(&self) -> f32 {
        self.latitude
    }
    fn longitude(&self) -> f32 {
        self.longitude
    }
    fn altitude(&self) -> i16 {
        0
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let point1 = Point { latitude: 51.301389, longitude: 6.953333 };
    let point2 = Point { latitude: 50.823194, longitude: 6.186389 };
    c.bench_function("haversine", |b| b.iter(|| haversine_distance(&point1, &point2)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
