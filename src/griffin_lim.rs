use rustfft::num_complex::Complex;

use crate::{
    fft::{fix_scale, Fft},
    float::Float,
};

pub fn griffin_lim<T: Float>(
    fft: &Fft<T>,
    iteration: usize,
    spectrum: &[Complex<T>],
) -> Vec<Complex<T>> {
    let mut buf = spectrum.to_vec();

    for _ in 0..iteration {
        fft.inverse(&mut buf);
        fft.forward(&mut buf);
        fix_scale(&mut buf);
        for i in 0..spectrum.len() {
            buf[i].re = spectrum[i].re;
        }
    }

    buf
}
