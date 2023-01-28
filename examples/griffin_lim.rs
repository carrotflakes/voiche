mod wav;

use rustfft::num_complex::Complex;
use voiche::{
    fft::{fix_scale, Fft},
    windows,
};

fn main() {
    let window_size = 1024;
    let window: Vec<f32> = windows::hann_window(window_size);
    let slide_size = window_size / 4;

    let fft = Fft::new(window_size);
    let transformer = Transformer::new(window, slide_size, fft);

    wav::wav_file_convert("gl", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| griffin_lim(&transformer, &buf))
            .collect()
    });
}

fn griffin_lim(transformer: &Transformer, buf: &Vec<f32>) -> Vec<f32> {
    let norms = transformer.forward_norm(buf);

    let mut angles: Vec<Vec<_>> = norms
        .iter()
        .map(|x| {
            x.iter()
                .map(|x| (x * 543.0 + 12.0) % std::f32::consts::TAU) // TODO
                .collect()
        })
        .collect();

    for _ in 0..32 {
        let buf = transformer.inverse(buf.len(), &norms, &angles);
        angles = transformer.forward_angle(&buf);
    }

    transformer.inverse(buf.len(), &norms, &angles)
}

pub struct Transformer {
    window: Vec<f32>,
    slide_size: usize,
    fft: Fft<f32>,
}

impl Transformer {
    pub fn new(window: Vec<f32>, slide_size: usize, fft: Fft<f32>) -> Self {
        Self {
            window,
            slide_size,
            fft,
        }
    }

    pub fn forward_norm(&self, buf: &[f32]) -> Vec<Vec<f32>> {
        self.forward(buf, |c| c.norm())
    }

    pub fn forward_angle(&self, buf: &[f32]) -> Vec<Vec<f32>> {
        self.forward(buf, |c| c.arg())
    }

    fn forward(&self, buf: &[f32], f: impl Fn(Complex<f32>) -> f32) -> Vec<Vec<f32>> {
        buf.windows(self.window.len())
            .step_by(self.slide_size)
            .map(|b| {
                let mut spec = b
                    .iter()
                    .zip(self.window.iter())
                    .map(|(x, y)| Complex::new(x * y, 0.0))
                    .collect();
                self.fft.forward(&mut spec);
                spec.into_iter().map(&f).collect()
            })
            .collect()
    }

    pub fn inverse(&self, size: usize, norms: &Vec<Vec<f32>>, angles: &Vec<Vec<f32>>) -> Vec<f32> {
        let output_scale = self.slide_size as f32 / self.window.iter().copied().sum::<f32>();
        let mut buf = vec![0.0; size];
        for (i, (norm, angle)) in norms.iter().zip(angles.iter()).enumerate() {
            let mut spec = norm
                .iter()
                .zip(angle.iter())
                .map(|(&n, &a)| Complex::from_polar(n, a))
                .collect();
            self.fft.inverse(&mut spec);
            fix_scale(&mut spec);
            for (j, (x, y)) in spec.iter().zip(self.window.iter()).enumerate() {
                buf[i * self.slide_size + j] += output_scale * x.re * y;
            }
        }
        buf
    }
}
