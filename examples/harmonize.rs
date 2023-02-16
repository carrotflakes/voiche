mod wav;

use std::iter::Sum;

use voiche::{
    api::retouch_spectrum, fft::Fft, pitch_detection, pitch_shift, transform::transform, windows,
    Float,
};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;

    wav::wav_file_convert("hrm", |sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                let process = harmonize(
                    windows::hann_window(window_size),
                    windows::trapezoid_window(window_size, window_size - slide_size),
                    slide_size,
                    sample_rate,
                );

                transform(window_size, slide_size, process, &buf)
            })
            .collect()
    });
}

pub fn harmonize<T: Float + Sum>(
    pre_window: Vec<T>,
    post_window: Vec<T>,
    slide_size: usize,
    sample_rate: u32,
) -> impl FnMut(&[T]) -> Vec<T> {
    let sample_rate = T::from(sample_rate).unwrap();
    let window_size = pre_window.len();
    let fft = Fft::new(window_size);
    let mut pitch_shift1 = pitch_shift::pitch_shifter(window_size);
    let mut pitch_shift2 = pitch_shift::pitch_shifter(window_size);
    let min_wavelength = sample_rate / T::from(440.0 * 5.0).unwrap();
    let peak_threshold = T::from(0.4).unwrap();

    move |buf: &[T]| {
        let b = buf.to_vec();

        retouch_spectrum(
            &fft,
            &pre_window,
            &post_window,
            slide_size,
            buf,
            |spectrum| {
                let nsdf = pitch_detection::compute_nsdf(&fft, &b);
                let peaks = pitch_detection::compute_peaks(&nsdf[..nsdf.len() / 2]);
                let peaks: Vec<_> = peaks.into_iter().filter(|p| min_wavelength < p.0).collect();
                let max_peak = peaks.iter().fold(T::zero(), |a, p| a.max(p.1));
                if peak_threshold < max_peak {
                    let t = T::from(0.9).unwrap();
                    let peak = peaks.iter().find(|p| max_peak * t <= p.1).unwrap().clone();
                    let wavelength = peak.0;
                    let freq = sample_rate / wavelength;
                    let nn = (freq / T::from(440.0).unwrap()).log2() * T::from(12.0).unwrap();
                    let nn_correct1 = ((nn * T::from(7.0 / 12.0).unwrap()).round()
                        / T::from(7.0 / 12.0).unwrap())
                    .round();
                    let nn_correct2 = (((nn * T::from(7.0 / 12.0).unwrap()).round()
                        - T::from(2.0).unwrap())
                        / T::from(7.0 / 12.0).unwrap())
                    .round();
                    let pitch1 = ((nn_correct1 - nn) / T::from(12.0).unwrap()).exp2();
                    let pitch2 = ((nn_correct2 - nn) / T::from(12.0).unwrap()).exp2();

                    let mut spec1 = spectrum.to_vec();
                    pitch_shift::process_spectrum(
                        slide_size,
                        &mut pitch_shift1,
                        pitch1,
                        &mut spec1,
                    );
                    let mut spec2 = spectrum.to_vec();
                    pitch_shift::process_spectrum(
                        slide_size,
                        &mut pitch_shift2,
                        pitch2,
                        &mut spec2,
                    );

                    for i in 0..spectrum.len() {
                        spectrum[i] = spec1[i] + spec2[i]
                    }
                }
            },
        )
    }
}
