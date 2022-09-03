pub use ndarray::prelude::*;
pub use num_traits::*;
pub use itertools::*;

pub type Idx = usize;
pub type IdxPair = (Idx, Idx);

pub const ROWS: Axis = Axis(0);
pub const COLS: Axis = Axis(1);