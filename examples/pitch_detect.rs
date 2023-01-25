mod wav;

use rustfft::num_complex::Complex;
use voiche::{apply_window, fft::Fft, windows};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (mut spec, bufs) = wav::load(&file);
    let sample_rate = spec.sample_rate;

    let start = std::time::Instant::now();
    let window_size = 1024 * 4;
    let window = windows::hann_window(window_size);
    let slide_size = window_size / 4;
    let fft = Fft::new(window_size);
    let peak_threshold = 0.7;

    let mut wavelength = sample_rate as f32 / 440.0;
    let mut osc = {
        let mut phase = 0.0f32;
        let mut g = 0.0;
        move |wavelength: f32, gain: f32| {
            g = g * 0.99 + gain * 0.01;
            phase = (phase + 1.0 / wavelength) % 1.0;
            (phase * std::f32::consts::TAU).sin() * g
            // (phase * 2.0 - 1.0)
        }
    };
    let converted: Vec<_> = bufs[0]
        .windows(window_size)
        .step_by(slide_size)
        .flat_map(|buf| {
            let buf: Vec<_> = apply_window(&window, buf.iter().copied()).collect();
            let nsdf = compute_nsdf(&fft, buf);
            let peaks = compute_peaks(window_size, &nsdf);

            let min_wavelength = sample_rate as f32 / (440.0 * 5.0);
            let peaks: Vec<_> = peaks.into_iter().filter(|p| min_wavelength < p.0).collect();

            let max_peak = peaks.iter().fold(0.0f32, |a, p| a.max(p.1));
            let mut gain = 0.0;
            if peak_threshold < max_peak {
                let peak = &peaks.iter().find(|p| max_peak * 0.9 <= p.1).unwrap();
                wavelength = peak.0;
                gain = peak.1;
            }

            let mut buf = vec![0.0; slide_size];
            for x in buf.iter_mut() {
                *x = osc(wavelength, gain.min(1.0));
            }
            buf
        })
        .collect();

    dbg!(start.elapsed());

    spec.channels = 2;
    wav::save(
        file.replace(".", "_pd."),
        spec,
        vec![bufs[0][0..converted.len()].to_vec(), converted],
    );
}

/// Normalized Square Difference Function (NSDF)
fn compute_nsdf(fft: &Fft<f32>, buf: Vec<f32>) -> Vec<f32> {
    let mut cmps: Vec<_> = buf.iter().copied().map(Complex::from).collect();
    fft.forward(&mut cmps);
    compute_nsdf_from_spectrum(fft, buf, cmps)
}

fn compute_nsdf_from_spectrum(
    fft: &Fft<f32>,
    buf: Vec<f32>,
    mut spectrum: Vec<Complex<f32>>,
) -> Vec<f32> {
    for x in &mut spectrum {
        *x = Complex::from(x.norm_sqr());
    }
    fft.inverse(&mut spectrum);

    let len = buf.len();
    let mut nsdf = vec![0.0; len];
    let mut m = f32::EPSILON;
    for i in 0..len {
        let inv = len - i - 1;
        m += buf[i].powi(2) + buf[inv].powi(2);
        nsdf[inv] = 2.0 * spectrum[inv].re / (m * len as f32);
    }

    nsdf
}

fn compute_peaks(window_size: usize, nsdf: &[f32]) -> Vec<(f32, f32)> {
    let mut peak = (0.0, 0.0);
    let mut peaks = vec![];
    let mut is_first = true;

    for i in 0..window_size / 2 {
        if nsdf[i + 1] < 0.0 {
            if 0.0 < peak.1 {
                peaks.push(peak);
                peak = (0.0, 0.0);
            }
            is_first = false;
            continue;
        }

        if !is_first && nsdf[i + 1] - nsdf[i] > 0.0 && nsdf[i + 2] - nsdf[i + 1] <= 0.0 {
            let t = 2.0 * (nsdf[i] - 2.0 * nsdf[i + 1] + nsdf[i + 2]);
            let d = (nsdf[i] - nsdf[i + 2]) / t;
            let c = nsdf[i + 1] - t * d * d / 4.0;
            if peak.1 < c {
                peak = (i as f32 + d, c);
            }
        }
    }
    peaks
}
