#[macro_use]
extern crate assert_approx_eq;

extern crate aeroscore;

#[test]
fn it_works() {
    let a: f64 = 1.0;
    assert_approx_eq!(a, 1.0);
}
