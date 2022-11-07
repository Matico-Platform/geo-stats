#![feature(test)]
extern crate test;

/// Utility to handle using rayons par iterator or not
macro_rules! cfg_into_iter {
    ($e: expr, $min_len: expr) => {{
        #[cfg(not(target_arch = "wasm32"))]
        let result = $e.into_par_iter().with_min_len($min_len);

        #[cfg(target_arch = "wasm32")]
        let result = $e.into_iter();

        result
    }};
    ($e: expr) => {{
        #[cfg(not(target_arch = "wasm32"))]
        let result = $e.into_par_iter();

        #[cfg(target_arch = "wasm32")]
        let result = $e.into_iter();

        result
    }};
}


pub mod lisa;
