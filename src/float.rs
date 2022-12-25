use rustfft::{num_traits, FftNum};

pub trait Float: FftNum + num_traits::Float + num_traits::FloatConst {}

impl<T: FftNum + num_traits::Float + num_traits::FloatConst> Float for T {}
