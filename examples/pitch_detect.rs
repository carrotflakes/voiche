mod wav;

use voiche::{fft::Fft, pitch_detection::pitch_detect, windows};

fn main() {
    let window_size = 1024 * 4;
    let window = windows::hann_window(window_size);
    let slide_size = window_size / 4;
    let fft = Fft::new(window_size);
    let peak_threshold = 0.7;

    wav::wav_file_convert("pd", |sample_rate, channels| {
        let mut wavelength = sample_rate as f32 / 440.0;
        let mut osc = {
            let mut phase = 0.0f32;
            let mut g = 0.0;
            move |wavelength: f32, gain: f32| {
                g = g * 0.99 + gain * 0.01;
                phase = (phase + 1.0 / wavelength) % 1.0;
                (phase * std::f32::consts::TAU).sin() * g
                // (phase * 2.0 - 1.0) * g
            }
        };

        let process = |buf| {
            let min_wavelength = sample_rate as f32 / (440.0 * 5.0);
            let peak = pitch_detect(&fft, &window, buf, min_wavelength, peak_threshold);
            let mut gain = 0.0;
            if let Some(peak) = peak {
                wavelength = peak.0;
                gain = peak.1;
            }

            let mut buf = vec![0.0; slide_size];
            for x in buf.iter_mut() {
                *x = osc(wavelength, gain.min(1.0));
            }
            buf
        };

        let converted: Vec<_> = channels[0]
            .windows(window_size)
            .step_by(slide_size)
            .flat_map(process)
            .collect();
        vec![channels[0][0..converted.len()].to_vec(), converted]
    });
}
