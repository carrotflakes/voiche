use std::f64::consts::TAU;

use rustfft::{
    num_complex::Complex,
    num_traits::{Num, One, Zero},
};

use crate::{fft::fill_right_part_of_spectrum, Float};

pub fn process_spectrum<T: Float>(
    slide_size: usize,
    pitch_shift: &mut impl FnMut(&[Complex<T>], T, usize) -> Vec<Complex<T>>,
    pitch: T,
    spectrum: &mut [Complex<T>],
) {
    let mut shifted_spectrum = pitch_shift(spectrum, pitch, slide_size);

    remove_aliasing(pitch, &mut shifted_spectrum);

    spectrum.copy_from_slice(&shifted_spectrum);
}

pub fn pitch_shifter<T: Float>(
    len: usize,
) -> impl FnMut(&[Complex<T>], T, usize) -> Vec<Complex<T>> {
    let mut prev_input_phases = vec![T::zero(); len];
    let mut prev_output_phases = vec![T::zero(); len];

    move |spectrum, pitch, slide_size| {
        let len = spectrum.len();

        let mut pre = vec![[T::zero(); 2]; len / 2 + 1];
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

        let mut shifted_spectrum = spectrum.to_vec();
        for i in 0..len / 2 + 1 {
            let shifted_bin = (T::from(i).unwrap() / pitch).round().to_usize().unwrap();
            let post = if shifted_bin > len / 2 {
                [T::zero(), T::zero()]
            } else {
                [pre[shifted_bin][0], pre[shifted_bin][1] * pitch]
            };

            let bin_deviation = post[1] - T::from(i).unwrap();
            let mut phase_diff =
                bin_deviation * T::from(TAU * slide_size as f64 / len as f64).unwrap();
            let bin_center_freq = T::from(TAU * i as f64 / len as f64).unwrap();
            phase_diff = phase_diff + bin_center_freq * T::from(slide_size).unwrap();

            let phase = wrap_phase(prev_output_phases[i] + phase_diff);
            shifted_spectrum[i] = Complex::from_polar(post[0], phase);
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

pub fn remove_aliasing<T: Num + Zero + One + Copy, S: Float>(pitch: S, buffer: &mut [T]) {
    let len = buffer.len();

    if pitch < S::one() {
        let nyquist = (S::from(len as f64 / 2.0).unwrap() * pitch)
            .round()
            .to_usize()
            .unwrap();
        buffer[nyquist..len - nyquist + 1].fill(T::zero());
    }
}
