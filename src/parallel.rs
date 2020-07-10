use cfg_if::*;

cfg_if! {
    if #[cfg(feature = "rayon")] {
        use rayon::slice;
        pub use rayon::prelude::*;
        pub fn opt_par_iter<T: Sync>(x: &[T]) -> slice::Iter<T> {
            x.par_iter()
        }

    } else {
        use std::slice;
        pub fn opt_par_iter<T>(x: &[T]) -> slice::Iter<T> {
            x.iter()
        }
    }
}
