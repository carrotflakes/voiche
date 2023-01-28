mod wav;

use rustfft::num_complex::Complex;
use voiche::{
    api, apply_window, apply_window_with_scale,
    fft::{fix_scale, Fft},
    overlapping_flatten, transform,
    windows::hann_window,
};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;

    wav::wav_file_convert("dgl", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                let window = hann_window(window_size);
                let fft = Fft::new(window_size);
                let mut gl = dynamic_griffin_lim(window.clone(), slide_size, 8);

                let process = move |buf: &[f32]| {
                    api::retouch_spectrum(&fft, &window, &window, slide_size, buf, &mut gl)
                };

                transform::transform(window_size, slide_size, process, &buf)
            })
            .collect()
    });
}

fn dynamic_griffin_lim(
    window: Vec<f32>,
    slide_size: usize,
    iterate: usize,
) -> impl FnMut(&mut [Complex<f32>]) {
    let len = window.len();
    let fft = Fft::new(len);
    let mut specs = vec![vec![[0.0, 0.0]; len]];

    move |spec: &mut [Complex<f32>]| {
        let prev = &specs[0];
        let phase_factor = std::f32::consts::TAU * slide_size as f32 / len as f32;
        specs.push(
            spec.iter()
                .enumerate()
                .map(|(i, x)| {
                    // Estimate phase from prev
                    let phase_diff = if i < len / 2 {
                        i as f32
                    } else {
                        i as f32 - len as f32
                    } * phase_factor;
                    let phase = prev[i][1] + phase_diff;
                    [x.norm(), phase]
                })
                .collect(),
        );

        for _ in 0..iterate {
            // Reconstruct waveform from spectrogram
            let waveform = reconstruct(&window, slide_size, &fft, &specs);

            // Update phases
            for (b, p) in waveform
                .windows(len)
                .step_by(slide_size)
                .zip(specs.iter_mut())
            {
                let mut spec = apply_window(&window, b.iter().copied())
                    .map(Complex::from)
                    .collect();
                fft.forward(&mut spec);
                for i in 0..spec.len() {
                    p[i][1] = spec[i].arg();
                }
            }
        }

        spec.iter_mut()
            .zip(specs[1].iter())
            .for_each(|(x, &[n, a])| *x = Complex::from_polar(n, a));
        specs.remove(0);
    }
}

fn reconstruct(
    window: &[f32],
    slide_size: usize,
    fft: &Fft<f32>,
    specs: &[Vec<[f32; 2]>],
) -> Vec<f32> {
    let overlap_size = window.len() - slide_size;
    let output_scale = slide_size as f32 / window.iter().copied().sum::<f32>();
    let mut output = Vec::with_capacity(overlap_size + specs.len() * slide_size);
    output.extend(vec![0.0; overlap_size]);
    for spec in specs {
        let mut spec = spec
            .iter()
            .map(|&[n, a]| Complex::from_polar(n, a))
            .collect();
        fft.inverse(&mut spec);
        fix_scale(&mut spec);

        overlapping_flatten::buffer_overlapping_write(
            overlap_size,
            &mut output,
            apply_window_with_scale(window, output_scale, spec.iter().map(|x| x.re)),
        );
    }
    output
}
