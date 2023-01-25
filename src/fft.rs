use std::sync::Arc;

use rustfft::{num_complex::Complex, FftNum};

#[derive(Clone)]
pub struct Fft<T: FftNum> {
    forward: Arc<dyn rustfft::Fft<T>>,
    inverse: Arc<dyn rustfft::Fft<T>>,
}

impl<T: FftNum> Fft<T> {
    pub fn new(size: usize) -> Self {
        let mut planner = rustfft::FftPlanner::new();
        Self {
            forward: planner.plan_fft_forward(size),
            inverse: planner.plan_fft_inverse(size),
        }
    }

    pub fn retouch_spectrum(&self, buf: &mut [T], mut process: impl FnMut(&mut [Complex<T>])) {
        let mut spec: Vec<_> = buf.iter().map(|&x| Complex::from(x)).collect();
        self.forward(&mut spec);
        process(&mut spec);
        self.inverse(&mut spec);
        fix_scale(&mut spec);
        buf.iter_mut().zip(spec.iter()).for_each(|(x, y)| *x = y.re);
    }

    pub fn forward(&self, buffer: &mut Vec<Complex<T>>) {
        self.forward.process(buffer);
    }

    pub fn inverse(&self, buffer: &mut Vec<Complex<T>>) {
        self.inverse.process(buffer);
    }
}

pub fn fix_scale<T: FftNum>(buf: &mut [Complex<T>]) {
    let scale = T::one() / T::from_usize(buf.len()).unwrap();
    for x in buf.iter_mut() {
        *x = *x * scale;
    }
}

pub fn fill_right_part_of_spectrum<T: FftNum>(spectrum: &mut [Complex<T>]) {
    let len = spectrum.len();

    for i in 1..len / 2 {
        spectrum[len - i] = spectrum[i].conj();
    }
}
