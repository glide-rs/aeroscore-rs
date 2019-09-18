#[macro_use] extern crate cfg_if;

extern crate failure;
extern crate flat_projection;
extern crate ord_subset;

#[cfg(feature = "rayon")]
extern crate rayon;

pub mod olc;
pub mod olc2;
pub mod olc3;
pub mod olc4;
