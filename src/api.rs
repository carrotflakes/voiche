use crate::{
    apply_window, apply_window_with_scale,
    fft::{self, Fft},
    float::Float,
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
