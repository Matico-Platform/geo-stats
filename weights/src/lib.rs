extern crate num_traits;

pub mod distance_weights;
pub mod queens_weights;
pub mod rook_weights;
mod utils;
pub mod weights;

pub use distance_weights::*;
pub use queens_weights::*;
pub use rook_weights::*;
pub use weights::*;
