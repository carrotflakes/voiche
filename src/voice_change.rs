use crate::{
    fft::{fill_right_part_of_spectrum, fix_scale, Fft},
    float::Float,
    pitch_shift::{pitch_shifter, remove_aliasing},
};
use rustfft::{num_complex::Complex, num_traits::Zero};

pub fn transform_processor<T: Float>(
    window_size: usize,
    slide_size: usize,
    envelope_order: usize,
    formant: T,
    pitch: T,
) -> impl FnMut(&[T]) -> Vec<T> {
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

pub fn process_spectrum<T: Float>(
    slide_size: usize,
    fft: &Fft<T>,
    pitch_shift: &mut impl FnMut(&[Complex<T>], T, usize) -> Vec<Complex<T>>,
    envelope_order: usize,
    formant: T,
    pitch: T,
    spectrum: &mut [Complex<T>],
) {
    assert!(0 < envelope_order && envelope_order < spectrum.len() / 2);

    let formant_expand_amount = T::from(2).unwrap().powf(formant);
    let pitch_change_amount = T::from(2).unwrap().powf(pitch);
    let len = spectrum.len();

    // formant shift
    let envelope = lift_spectrum(&fft, spectrum, |b| {
        b[envelope_order..len - envelope_order + 1].fill(Complex::zero());
    });
    let shifted_envelope = formant_shift(envelope, formant_expand_amount);

    // pitch shift
    let shifted_spectrum = pitch_shift(spectrum, pitch_change_amount, slide_size);

    // extract fine structure
    let mut fine_structure = lift_spectrum(&fft, &shifted_spectrum, |b| {
        b[..envelope_order].fill(Complex::zero());
        b[len - envelope_order + 1..].fill(Complex::zero());
    });

    remove_aliasing(pitch_change_amount, &mut fine_structure);

    for i in 0..len / 2 + 1 {
        let amp = (shifted_envelope[i] + fine_structure[i]).exp();
        let phase = shifted_spectrum[i].arg();
        spectrum[i] = Complex::from_polar(amp, phase);
    }

    fill_right_part_of_spectrum(spectrum);
}

pub fn formant_shift<T: Float>(envelope: Vec<T>, formant_expand_amount: T) -> Vec<T> {
    let len = envelope.len();
    let negative = T::from(-1000.0).unwrap();

    let mut new_envelope = vec![T::zero(); len];
    for i in 0..len / 2 + 1 {
        let j_f32 = T::from(i).unwrap() / formant_expand_amount;
        let j = j_f32.floor().to_usize().unwrap();
        let l = if j <= len / 2 { envelope[j] } else { negative };
        let r = if j + 1 <= len / 2 {
            envelope[j + 1]
        } else {
            negative
        };
        let x = j_f32 - T::from(j).unwrap();
        new_envelope[i] = (T::one() - x) * l + x * r;
    }
    for i in 1..len / 2 + 1 {
        new_envelope[len - i] = new_envelope[i];
    }
    new_envelope
}

pub fn lift_spectrum<T: Float>(
    fft: &Fft<T>,
    spectrum: &[Complex<T>],
    mut process: impl FnMut(&mut Vec<Complex<T>>),
) -> Vec<T> {
    let mut cepstrum: Vec<_> = spectrum
        .iter()
        .map(|&x| Complex::new((x.norm() + T::epsilon()).ln(), T::zero()))
        .collect();

    fft.inverse(&mut cepstrum);

    process(&mut cepstrum);

    let mut envelope = cepstrum;
    fft.forward(&mut envelope);
    fix_scale(&mut envelope);

    envelope.into_iter().map(|x| x.re).collect()
}
