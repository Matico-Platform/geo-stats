extern crate num_traits;

pub mod weights;
pub mod euclidan_weights;

pub use weights::*;
pub use euclidan_weights::*;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
