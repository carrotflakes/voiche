pub mod api;
pub mod fft;
pub mod float;
pub mod overlapping_flatten;
pub mod pitch_detection;
pub mod pitch_shift;
pub mod transform;
pub mod voice_change;
pub mod windows;

pub use float::Float;
pub use rustfft::{self, num_complex, num_traits};

pub fn apply_window<'a, T: rustfft::num_traits::Float>(
    window: &'a [T],
    iter: impl Iterator<Item = T> + 'a,
) -> impl Iterator<Item = T> + 'a {
    iter.zip(window.iter()).map(|(x, &y)| x * y)
}

pub fn apply_window_with_scale<'a, T: rustfft::num_traits::Float>(
    window: &'a [T],
    scale: T,
    iter: impl Iterator<Item = T> + 'a,
) -> impl Iterator<Item = T> + 'a {
    iter.zip(window.iter()).map(move |(x, &y)| x * y * scale)
}
