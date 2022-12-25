use rustfft::num_traits;

pub fn hann_window<T: num_traits::Float + num_traits::FloatConst>(size: usize) -> Vec<T> {
    (0..size)
        .map(|i| {
            T::from(0.5).unwrap()
                * (T::one() - (T::from(i).unwrap() * T::TAU() / T::from(size).unwrap()).cos())
        })
        .collect()
}
