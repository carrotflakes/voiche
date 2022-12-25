use std::f64::consts::TAU;

use rustfft::{
    num_complex::Complex,
    num_traits::{Num, One, Zero},
};

use crate::{
    fft::{fill_right_part_of_spectrum, Fft},
    float::Float,
};

pub fn transform_processor<T: Float>(
    window_size: usize,
    slide_size: usize,
    pitch: T,
) -> impl FnMut(&[T]) -> Vec<T> {
    let fft = Fft::new(window_size);
    let mut pitch_shift = pitch_shifter(window_size);

    move |buf| {
        fft.retouch_spectrum(buf, |spectrum| {
            process_spectrum(slide_size, &mut pitch_shift, pitch, spectrum);
        })
    }
}

pub fn process_spectrum<T: Float>(
    slide_size: usize,
    pitch_shift: &mut impl FnMut(&[Complex<T>], T, usize) -> Vec<Complex<T>>,
    pitch: T,
    spectrum: &mut [Complex<T>],
) {
    let pitch_change_amount = T::from_i32(2).unwrap().powf(pitch);
    let len = spectrum.len();

    let mut shifted_spectrum = pitch_shift(spectrum, pitch_change_amount, slide_size);

    remove_aliasing(pitch_change_amount, &mut shifted_spectrum);

    spectrum[..len / 2 + 1].copy_from_slice(&shifted_spectrum[..len / 2 + 1]);

    fill_right_part_of_spectrum(spectrum);
}

pub fn pitch_shifter<T: Float>(
    len: usize,
) -> impl FnMut(&[Complex<T>], T, usize) -> Vec<Complex<T>> {
    let mut prev_input_phases = vec![T::zero(); len];
    let mut prev_output_phases = vec![T::zero(); len];

    move |spectrum, pitch_change_amount, slide_size| {
        let len = spectrum.len();

        let mut pre = vec![[T::zero(); 2]; len];
        for i in 0..len / 2 + 1 {
            let (norm, phase) = spectrum[i].to_polar();
            let bin_center_freq = T::from(TAU * i as f64 / len as f64).unwrap();

            let phase_diff =
                phase - prev_input_phases[i] - bin_center_freq * T::from(slide_size).unwrap();
            let phase_diff = wrap_phase(phase_diff);
            prev_input_phases[i] = phase;
            let bin_deviation =
                phase_diff * T::from(len as f64 / (slide_size as f64 * TAU)).unwrap();

            pre[i] = [norm, T::from(i).unwrap() + bin_deviation];
        }

        let mut post = vec![[T::zero(); 2]; len];
        for i in 0..len / 2 + 1 {
            let shifted_bin = (T::from(i).unwrap() / pitch_change_amount)
                .round()
                .to_usize()
                .unwrap();
            if shifted_bin > len / 2 {
                break;
            }
            post[i] = [
                pre[shifted_bin][0],
                pre[shifted_bin][1] * pitch_change_amount,
            ];
        }

        let mut shifted_spectrum = spectrum.to_vec();
        for i in 0..len / 2 + 1 {
            let bin_deviation = post[i][1] - T::from(i).unwrap();
            let mut phase_diff =
                bin_deviation * T::from(TAU * slide_size as f64 / len as f64).unwrap();
            let bin_center_freq = T::from(TAU * i as f64 / len as f64).unwrap();
            phase_diff = phase_diff + bin_center_freq * T::from(slide_size).unwrap();

            let phase = wrap_phase(prev_output_phases[i] + phase_diff);
            shifted_spectrum[i] = Complex::from_polar(post[i][0], phase);
            prev_output_phases[i] = phase;
        }

        fill_right_part_of_spectrum(&mut shifted_spectrum);

        shifted_spectrum
    }
}

pub fn wrap_phase<T: Float>(phase: T) -> T {
    if phase >= T::zero() {
        (phase + T::PI()) % T::TAU() - T::PI()
    } else {
        (phase - T::PI()) % T::TAU() + T::PI()
    }
}

pub fn remove_aliasing<T: Num + Zero + One + Copy, S: Float>(
    pitch_change_amount: S,
    fine_structure: &mut [T],
) {
    let len = fine_structure.len();

    if pitch_change_amount < S::one() {
        let nyquist = (S::from(len as f64 / 2.0).unwrap() * pitch_change_amount)
            .round()
            .to_usize()
            .unwrap();
        fine_structure[nyquist..len / 2].fill(T::zero());

        for i in 1..len / 2 {
            fine_structure[len - i] = fine_structure[i];
        }
    }
}
