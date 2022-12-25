use rustfft::num_traits;

pub fn rectangular_window<T: num_traits::Float + num_traits::FloatConst>(size: usize) -> Vec<T> {
    vec![T::one(); size]
}

pub fn hann_window<T: num_traits::Float + num_traits::FloatConst>(size: usize) -> Vec<T> {
    (0..size)
        .map(|i| {
            T::from(0.5).unwrap()
                * (T::one() - (T::from(i).unwrap() * T::TAU() / T::from(size).unwrap()).cos())
        })
        .collect()
}

pub fn hamming_window<T: num_traits::Float + num_traits::FloatConst>(size: usize) -> Vec<T> {
    let a = T::from(25.0 / 46.0).unwrap();
    (0..size)
        .map(|i| {
            a - (T::one() - a) * (T::from(i).unwrap() * T::TAU() / T::from(size).unwrap()).cos()
        })
        .collect()
}

pub fn blackman_window<T: num_traits::Float + num_traits::FloatConst>(
    alpha: T,
    size: usize,
) -> Vec<T> {
    let a0 = (T::one() - alpha) / T::from(2).unwrap();
    let a1 = T::one() / T::from(2).unwrap();
    let a2 = alpha / T::from(2).unwrap();
    (0..size)
        .map(|i| {
            a0 - a1 * (T::from(i).unwrap() * T::TAU() / T::from(size).unwrap()).cos()
                + a2 * (T::from(2 * i).unwrap() * T::TAU() / T::from(size).unwrap()).cos()
        })
        .collect()
}

pub fn blackman_window_default<T: num_traits::Float + num_traits::FloatConst>(
    size: usize,
) -> Vec<T> {
    blackman_window(T::from(0.16).unwrap(), size)
}

#[test]
fn test() {
    dbg!(hann_window::<f32>(10));
    dbg!(hamming_window::<f32>(10));
    dbg!(blackman_window::<f32>(0.16, 10));
}
