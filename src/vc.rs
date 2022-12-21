use std::sync::Arc;

use rustfft::num_complex::{Complex, Complex32};

pub fn vc(
    buf: &[f32],
    mut process: impl FnMut(&[f32]) -> Vec<f32>,
    window_size: usize,
    slide_size: usize,
) -> Vec<f32> {
    let window: Vec<_> = (0..window_size)
        .map(|i| {
            let omega = std::f32::consts::TAU / window_size as f32;
            0.5 * (1.0 - (omega * i as f32).cos())
        })
        .collect();

    let mut output = vec![0.0; buf.len()];
    let output_scale = slide_size as f32 / window.iter().sum::<f32>();

    for i in 0..buf.len() / slide_size {
        let i = i * slide_size;
        let mut b: Vec<_> = buf[i..]
            .iter()
            .zip(window.iter())
            .map(|(x, y)| x * y)
            .collect();
        b.resize(window_size, 0.0);
        let b = process(&b);
        let b: Vec<_> = b.into_iter().enumerate().map(|(i, x)| x * window[i] * output_scale).collect();
        for (x, y) in output[i..].iter_mut().zip(b.iter()) {
            *x += y;
        }
    }
    output
}

pub fn process_nop(buf: &[f32]) -> Vec<f32> {
    buf.to_vec()
}

pub fn process_rev(buf: &[f32]) -> Vec<f32> {
    let mut buf = buf.to_vec();
    buf.reverse();
    buf
}

pub struct Fft {
    pub forward: Arc<dyn rustfft::Fft<f32>>,
    pub inverse: Arc<dyn rustfft::Fft<f32>>,
}

impl Fft {
    pub fn new(size: usize) -> Self {
        let mut planner = rustfft::FftPlanner::new();
        Self {
            forward: planner.plan_fft_forward(size),
            inverse: planner.plan_fft_inverse(size),
        }
    }

    pub fn process(
        &self,
        buf: &[f32],
        mut process: impl FnMut(&Fft, &mut Vec<Complex32>),
    ) -> Vec<f32> {
        let mut buf: Vec<_> = buf.iter().map(|&x| Complex::new(x, 0.0)).collect();
        self.forward.process(&mut buf);
        process(self, &mut buf);
        self.inverse.process(&mut buf);
        let scale = 1.0 / buf.len() as f32;
        buf.iter().map(|x| x.re * scale).collect()
    }
}

pub fn power(buf: &[f32]) -> f32 {
    (buf.iter().map(|&x| x.powi(2)).sum::<f32>() / buf.len() as f32).sqrt()
}
