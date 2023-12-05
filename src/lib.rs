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

/// Resample with quadratic interpolation.
pub fn resample<T: float::Float>(buf: &[T], rate: T) -> Vec<T> {
    let mut output = vec![
        T::zero();
        (T::from(buf.len()).unwrap() * rate)
            .ceil()
            .to_usize()
            .unwrap()
    ];

    for i in 0..output.len() {
        let p = T::from(i).unwrap() / rate;
        let j = p.to_usize().unwrap();

        let x = buf[j];
        let y = buf.get(j + 1).copied().unwrap_or(T::zero());
        let z = buf.get(j + 2).copied().unwrap_or(T::zero());

        output[i] = y - (x - z) * p.fract() / T::from(4.0).unwrap();
    }

    output
}
