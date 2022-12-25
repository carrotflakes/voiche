pub mod fft;
pub mod float;
pub mod pitch_shift;
pub mod transform;
pub mod voice_change;
pub mod windows;

pub fn power<T: float::Float + std::iter::Sum>(buf: &[T]) -> T {
    (buf.iter().map(|&x| x.powi(2)).sum::<T>() / T::from(buf.len()).unwrap()).sqrt()
}
