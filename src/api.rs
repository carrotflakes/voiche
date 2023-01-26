use crate::{
    apply_window, apply_window_with_scale,
    fft::{self, Fft},
    float::Float,
    pitch_detection,
    pitch_shift::{self, pitch_shifter},
    voice_change,
};

pub fn pitch_shift<T: Float + std::iter::Sum>(
    pre_window: Vec<T>,
    post_window: Vec<T>,
    slide_size: usize,
    pitch: T,
) -> impl FnMut(&[T]) -> Vec<T> {
    assert_eq!(pre_window.len(), post_window.len());

    let window_size = pre_window.len();
    let fft = Fft::new(window_size);
    let mut pitch_shift = pitch_shifter(window_size);

    move |buf| {
        retouch_spectrum(
            &fft,
            &pre_window,
            &post_window,
            slide_size,
            &buf,
            |spectrum| {
                pitch_shift::process_spectrum(slide_size, &mut pitch_shift, pitch, spectrum);
            },
        )
    }
}

pub fn voice_change<T: Float + std::iter::Sum>(
    pre_window: Vec<T>,
    post_window: Vec<T>,
    slide_size: usize,
    envelope_order: usize,
    formant: T,
    pitch: T,
) -> impl FnMut(&[T]) -> Vec<T> {
    assert_eq!(pre_window.len(), post_window.len());

    let window_size = pre_window.len();
    let fft = Fft::new(window_size);
    let mut pitch_shift = pitch_shifter(window_size);

    move |buf| {
        retouch_spectrum(
            &fft,
            &pre_window,
            &post_window,
            slide_size,
            &buf,
            |spectrum| {
                voice_change::process_spectrum(
                    slide_size,
                    &fft,
                    &mut pitch_shift,
                    envelope_order,
                    formant,
                    pitch,
                    spectrum,
                );
            },
        )
    }
}

pub fn pitch_correct<T: Float + std::iter::Sum, F: FnMut(T) -> T>(
    pre_window: Vec<T>,
    post_window: Vec<T>,
    slide_size: usize,
    sample_rate: u32,
    mut pitch_fn: F,
) -> impl FnMut(&[T]) -> Vec<T> {
    let sample_rate = T::from(sample_rate).unwrap();
    let window_size = pre_window.len();
    let fft = Fft::new(window_size);
    let mut pitch_shift = pitch_shift::pitch_shifter(window_size);
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
                let pitch = if peak_threshold < max_peak {
                    let t = T::from(0.9).unwrap();
                    let peak = peaks.iter().find(|p| max_peak * t <= p.1).unwrap().clone();
                    let wavelength = peak.0;
                    let freq = sample_rate / wavelength;
                    pitch_fn(freq)
                } else {
                    T::zero()
                };
                pitch_shift::process_spectrum(slide_size, &mut pitch_shift, pitch, spectrum);
            },
        )
    }
}

pub fn retouch_spectrum<T: Float + std::iter::Sum>(
    fft: &Fft<T>,
    pre_window: &[T],
    post_window: &[T],
    slide_size: usize,
    buf: &[T],
    mut process: impl FnMut(&mut [rustfft::num_complex::Complex<T>]),
) -> Vec<T> {
    let mut spec: Vec<_> = apply_window(pre_window, buf.iter().copied())
        .map(rustfft::num_complex::Complex::from)
        .collect();
    fft.forward(&mut spec);
    process(&mut spec);
    fft.inverse(&mut spec);
    fft::fix_scale(&mut spec);
    let output_scale = T::from(slide_size).unwrap() / post_window.iter().copied().sum::<T>();
    apply_window_with_scale(post_window, output_scale, spec.iter().map(|x| x.re)).collect()
}
