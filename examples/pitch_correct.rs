mod wav;

use voiche::{fft::Fft, pitch_detection, pitch_shift, transform::transform, windows};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, bufs) = wav::load(&file);
    let sample_rate = spec.sample_rate;
    dbg!(wav::power(&bufs[0]));

    let start = std::time::Instant::now();
    let window_size = 1024;
    let window = windows::hann_window(window_size);
    let slide_size = window_size / 4;

    let bufs: Vec<_> = bufs
        .iter()
        .map(|buf| {
            let process = {
                let fft = Fft::new(window_size);
                let mut pitch_shift = pitch_shift::pitch_shifter(window_size);
                let min_wavelength = sample_rate as f32 / (440.0 * 5.0);
                let peak_threshold = 0.4;

                move |buf: &mut [f32]| {
                    let b = buf.to_vec();

                    fft.retouch_spectrum(buf, |spectrum| {
                        let nsdf = pitch_detection::compute_nsdf(&fft, &b);
                        let peaks = pitch_detection::compute_peaks(&nsdf[..nsdf.len() / 2]);
                        let peaks: Vec<_> =
                            peaks.into_iter().filter(|p| min_wavelength < p.0).collect();
                        let max_peak = peaks.iter().fold(0.0f32, |a, p| a.max(p.1));
                        let pitch = if peak_threshold < max_peak {
                            let peak = peaks
                                .iter()
                                .find(|p| max_peak * 0.9 <= p.1)
                                .unwrap()
                                .clone();
                            let wavelength = peak.0;
                            let freq = sample_rate as f32 / wavelength;
                            let nn = (freq / 440.0).log2() * 12.0;
                            // approximately scale
                            let nn_correct = ((nn * (7.0 / 12.0)).round() / (7.0 / 12.0)).round();
                            -(nn - nn_correct) / 12.0
                            // -(freq / 220.0).log2()
                        } else {
                            0.0
                        };
                        pitch_shift::process_spectrum(
                            slide_size,
                            &mut pitch_shift,
                            pitch,
                            spectrum,
                        );
                    })
                }
            };
            transform(slide_size, window.clone(), process, buf)
        })
        .collect();

    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_pc."), spec, bufs);
}
