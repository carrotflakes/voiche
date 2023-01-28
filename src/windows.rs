use crate::num_traits::{Float, FloatConst};

pub fn rectangular_window<T: Float + FloatConst>(size: usize) -> Vec<T> {
    vec![T::one(); size]
}

pub fn trapezoid_window<T: Float + FloatConst>(size: usize, sleeve: usize) -> Vec<T> {
    let sleeve_f = T::from(sleeve + 1).unwrap();
    let size_f = T::from(size).unwrap();
    (0..size)
        .map(|i| {
            let i_f = T::from(i).unwrap();
            ((i_f + T::one()) / sleeve_f)
                .min((size_f - i_f) / sleeve_f)
                .min(T::one())
        })
        .collect()
}

pub fn hann_window<T: Float + FloatConst>(size: usize) -> Vec<T> {
    (0..size)
        .map(|i| {
            T::from(0.5).unwrap()
                * (T::one() - (T::from(i).unwrap() * T::TAU() / T::from(size).unwrap()).cos())
        })
        .collect()
}

pub fn hamming_window<T: Float + FloatConst>(size: usize) -> Vec<T> {
    let a = T::from(25.0 / 46.0).unwrap();
    (0..size)
        .map(|i| {
            a - (T::one() - a) * (T::from(i).unwrap() * T::TAU() / T::from(size).unwrap()).cos()
        })
        .collect()
}

pub fn blackman_window<T: Float + FloatConst>(alpha: T, size: usize) -> Vec<T> {
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

pub fn blackman_window_default<T: Float + FloatConst>(size: usize) -> Vec<T> {
    blackman_window(T::from(0.16).unwrap(), size)
}

#[test]
fn test() {
    dbg!(trapezoid_window::<f32>(8, 0));
    dbg!(trapezoid_window::<f32>(8, 3));
    dbg!(trapezoid_window::<f32>(8, 4));
    dbg!(trapezoid_window::<f32>(8, 5));

    dbg!(hann_window::<f32>(10));
    dbg!(hamming_window::<f32>(10));
    dbg!(blackman_window::<f32>(0.16, 10));
}
