#[macro_use]
extern crate criterion;

extern crate aeroscore;
extern crate igc;

use criterion::Criterion;
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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("olc_classic", |b| b.iter(|| {
        let fixes = include_str!("../tests/fixtures/2017-08-14-fla-6ng-01.igc")
            .lines()
            .filter(|l| l.starts_with('B'))
            .map(|line| Point(igc::parse_fix(&line)))
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
