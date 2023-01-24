mod wav;

use rustfft::num_complex::Complex;
use voiche::{fft::Fft, transform::transform, windows};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (mut spec, bufs) = wav::load(&file);

    let start = std::time::Instant::now();
    let window_size = 1024 * 4;
    let window = windows::hann_window(window_size);
    let slide_size = window_size / 4;
    let fft = Fft::new(window_size);

    let mut pitch = 40.0;
    let converted = transform(
        slide_size,
        window.clone(),
        |buf| {
            let mut spec: Vec<_> = buf.iter().map(|&x| Complex::from(x)).collect();
            fft.forward(&mut spec);

            spec.iter_mut()
                .for_each(|x| *x = Complex::from(x.norm_sqr()));
            fft.inverse(&mut spec);

            let mut nsdf = vec![0.0; window_size];
            let mut m = f32::EPSILON;
            for i in 0..window_size {
                let inv = window_size - i - 1;
                m += buf[i].powi(2) + buf[inv].powi(2);

                nsdf[inv] = 2.0 * spec[inv].re / window_size as f32 / m;
            }

            let mut peak = (0.0, 0.0);
            let mut peaks = vec![];
            for i in window_size / 128..window_size / 2 {
                if nsdf[i + 1] < 0.0 {
                    if 0.0 < peak.0 {
                        peaks.push(peak);
                        peak = (0.0, 0.0);
                    }
                    continue;
                }
                if nsdf[i + 1] - nsdf[i] > 0.0 && nsdf[i + 2] - nsdf[i + 1] <= 0.0 {
                    let t = 2.0 * (nsdf[i] - 2.0 * nsdf[i + 1] + nsdf[i + 2]);
                    let d = (nsdf[i] - nsdf[i + 2]) / t;
                    let c = nsdf[i + 1] - t * d * d / 4.0;
                    if peak.0 < c {
                        peak = (c, i as f32 + d);
                    }
                }
            }

            let max_peak = peaks.iter().fold(0.0f32, |a, p| a.max(p.0));
            let mut gain = 0.0;
            if 0.3 < max_peak {
                let peak = &peaks.iter().find(|p| max_peak * 0.9 <= p.0).unwrap();
                pitch = peak.1;
                gain = peak.0;
            }
            // println!("{} {}", n, pitch);
            // if (0.3 < max.0) {
            //     println!("{:?}", &nsdf);
            //     panic!();
            // }
            let mut phase = 0.0f32;
            for x in buf {
                *x = (phase * std::f32::consts::TAU).sin() * gain;
                // *x = (phase * 2.0 - 1.0) * gain;
                phase = (phase + 1.0 / pitch as f32) % 1.0;
            }
        },
        &bufs[0],
    );

    dbg!(start.elapsed());

    spec.channels = 2;
    wav::save(
        file.replace(".", "_pd."),
        spec,
        vec![bufs[0].clone(), converted],
    );
}
