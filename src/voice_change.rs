use crate::{
    fft::{fill_right_part_of_spectrum, fix_scale, Fft},
    pitch_shift::{pitch_shifter, remove_aliasing},
};
use rustfft::{num_complex::Complex32, num_traits::Zero};

pub fn transform_processor(
    window_size: usize,
    slide_size: usize,
    envelope_order: usize,
    formant: f32,
    pitch: f32,
) -> impl FnMut(&[f32]) -> Vec<f32> {
    let fft = Fft::new(window_size);
    let mut pitch_shift = pitch_shifter(window_size);

    move |buf| {
        fft.retouch_spectrum(buf, |spectrum| {
            process_spectrum(
                slide_size,
                &fft,
                &mut pitch_shift,
                envelope_order,
                formant,
                pitch,
                spectrum,
            );
        })
    }
}

pub fn process_spectrum(
    slide_size: usize,
    fft: &Fft,
    pitch_shift: &mut impl FnMut(
        &[rustfft::num_complex::Complex<f32>],
        f32,
        usize,
    ) -> Vec<rustfft::num_complex::Complex<f32>>,
    envelope_order: usize,
    formant: f32,
    pitch: f32,
    spectrum: &mut [rustfft::num_complex::Complex<f32>],
) {
    assert!(0 < envelope_order && envelope_order < spectrum.len() / 2);

    let formant_expand_amount = 2.0f32.powf(formant);
    let pitch_change_amount = 2.0f32.powf(pitch);
    let len = spectrum.len();

    // formant shift
    let envelope = lift_spectrum(&fft, spectrum, |b| {
        b[envelope_order..len - envelope_order + 1].fill(Complex32::zero());
    });
    let shifted_envelope = formant_shift(envelope, formant_expand_amount);

    // pitch shift
    let shifted_spectrum = pitch_shift(spectrum, pitch_change_amount, slide_size);

    // extract fine structure
    let mut fine_structure = lift_spectrum(&fft, &shifted_spectrum, |b| {
        b[..envelope_order].fill(Complex32::zero());
        b[len - envelope_order + 1..].fill(Complex32::zero());
    });

    remove_aliasing(pitch_change_amount, &mut fine_structure);

    for i in 0..len / 2 + 1 {
        let amp = (shifted_envelope[i] + fine_structure[i]).exp();
        let phase = shifted_spectrum[i].arg();
        spectrum[i] = Complex32::from_polar(amp, phase);
    }

    fill_right_part_of_spectrum(spectrum);
}

pub fn formant_shift(envelope: Vec<f32>, formant_expand_amount: f32) -> Vec<f32> {
    let len = envelope.len();

    let mut new_envelope = vec![0.0; len];
    for i in 0..len / 2 + 1 {
        let j_f32 = i as f32 / formant_expand_amount;
        let j = j_f32.floor() as usize;
        let l = if j <= len / 2 { envelope[j] } else { -1000.0 };
        let r = if j + 1 <= len / 2 {
            envelope[j + 1]
        } else {
            -1000.0
        };
        let x = j_f32 - j as f32;
        new_envelope[i] = (1.0 - x) * l + x * r;
    }
    for i in 1..len / 2 + 1 {
        new_envelope[len - i] = new_envelope[i];
    }
    new_envelope
}

pub fn lift_spectrum(
    fft: &Fft,
    spectrum: &[Complex32],
    mut process: impl FnMut(&mut Vec<Complex32>),
) -> Vec<f32> {
    let mut cepstrum: Vec<_> = spectrum
        .iter()
        .map(|&x| Complex32::new((x.norm() + std::f32::EPSILON).ln(), 0.0))
        .collect();

    fft.inverse(&mut cepstrum);

    process(&mut cepstrum);

    let mut envelope = cepstrum;
    fft.forward(&mut envelope);
    fix_scale(&mut envelope);

    envelope.into_iter().map(|x| x.re).collect()
}
