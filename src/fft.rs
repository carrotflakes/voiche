use std::sync::Arc;

use rustfft::num_complex::Complex32;

pub struct Fft {
    forward: Arc<dyn rustfft::Fft<f32>>,
    inverse: Arc<dyn rustfft::Fft<f32>>,
}

impl Fft {
    pub fn new(size: usize) -> Self {
        let mut planner = rustfft::FftPlanner::new();
        Self {
            forward: planner.plan_fft_forward(size),
            inverse: planner.plan_fft_inverse(size),
        }
    }

    pub fn retouch_spectrum(
        &self,
        buf: &[f32],
        mut process: impl FnMut(&mut [Complex32]),
    ) -> Vec<f32> {
        let mut buf: Vec<_> = buf.iter().map(|&x| Complex32::new(x, 0.0)).collect();
        self.forward(&mut buf);
        process(&mut buf);
        self.inverse(&mut buf);
        fix_scale(&mut buf);
        buf.iter().map(|x| x.re).collect()
    }

    pub fn forward(&self, buffer: &mut Vec<Complex32>) {
        self.forward.process(buffer);
    }

    pub fn inverse(&self, buffer: &mut Vec<Complex32>) {
        self.inverse.process(buffer);
    }
}

pub fn fix_scale(buf: &mut [Complex32]) {
    let scale = 1.0 / buf.len() as f32;
    for x in buf.iter_mut() {
        *x *= scale;
    }
}

pub fn fill_right_part_of_spectrum(spectrum: &mut [Complex32]) {
    let len = spectrum.len();

    for i in 1..len / 2 {
        spectrum[len - i] = spectrum[i].conj();
    }
}
