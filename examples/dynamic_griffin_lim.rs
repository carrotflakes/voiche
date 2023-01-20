mod wav;

use rustfft::num_complex::Complex;
use voiche::{
    apply_window, apply_window_with_scale,
    fft::{fix_scale, Fft},
    transform,
    windows::hann_window,
};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, bufs) = wav::load(&file);
    dbg!(wav::power(&bufs[0]));

    let start = std::time::Instant::now();
    let window_size = 1024;
    let slide_size = window_size / 4;
    let window = hann_window(window_size);

    let bufs: Vec<_> = bufs
        .iter()
        .map(|buf| {
            let mut transformer = transform::Transformer::new(
                window.clone(),
                slide_size,
                dynamic_griffin_lim(window.clone(), slide_size, 8),
            );
            transformer.input_slice(&buf);
            let mut buf = Vec::new();
            transformer.finish(&mut buf);
            buf
        })
        .collect();

    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_dgl."), spec, bufs);
}

fn dynamic_griffin_lim(
    window: Vec<f32>,
    slide_size: usize,
    iterate: usize,
) -> impl FnMut(&mut [f32]) {
    let len = window.len();
    let fft = Fft::new(len);
    let mut specs = vec![vec![[0.0, 0.0]; len]];

    move |buf: &mut [f32]| {
        // Get spectrum from input
        let mut spec = apply_window(&window, buf.iter().copied())
            .map(Complex::from)
            .collect();
        fft.forward(&mut spec);
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

        // Reconstruct wavform from spectrum
        let mut spec = specs[1]
            .iter()
            .map(|&[n, a]| Complex::from_polar(n, a))
            .collect();
        fft.inverse(&mut spec);
        fix_scale(&mut spec);
        buf.copy_from_slice(&spec.iter().map(|x| x.re).collect::<Vec<_>>());
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

        transform::buffer_overlapping_write(
            overlap_size,
            &mut output,
            apply_window_with_scale(window, output_scale, spec.iter().map(|x| x.re)),
        );
    }
    output
}
