use std::f32::consts::{PI, TAU};

use rustfft::{
    num_complex::Complex32,
    num_traits::{Num, One, Zero},
};

use crate::fft::{fill_right_part_of_spectrum, Fft};

pub fn transform_processor(
    window_size: usize,
    slide_size: usize,
    pitch: f32,
) -> impl FnMut(&[f32]) -> Vec<f32> {
    let fft = Fft::new(window_size);
    let mut pitch_shift = pitch_shifter(window_size);

    move |buf| {
        fft.retouch_spectrum(buf, |spectrum| {
            process_spectrum(slide_size, &mut pitch_shift, pitch, spectrum);
        })
    }
}

pub fn process_spectrum(
    slide_size: usize,
    pitch_shift: &mut impl FnMut(
        &[rustfft::num_complex::Complex<f32>],
        f32,
        usize,
    ) -> Vec<rustfft::num_complex::Complex<f32>>,
    pitch: f32,
    spectrum: &mut [rustfft::num_complex::Complex<f32>],
) {
    let pitch_change_amount = 2.0f32.powf(pitch);
    let len = spectrum.len();

    let mut shifted_spectrum = pitch_shift(spectrum, pitch_change_amount, slide_size);

    remove_aliasing(pitch_change_amount, &mut shifted_spectrum);

    spectrum[..len / 2 + 1].copy_from_slice(&shifted_spectrum[..len / 2 + 1]);

    fill_right_part_of_spectrum(spectrum);
}

pub fn pitch_shifter(len: usize) -> impl FnMut(&[Complex32], f32, usize) -> Vec<Complex32> {
    let mut prev_input_phases = vec![0.0; len];
    let mut prev_output_phases = vec![0.0; len];

    move |spectrum, pitch_change_amount, slide_size| {
        let len = spectrum.len();

        let mut pre = vec![[0.0; 2]; len];
        for i in 0..len / 2 + 1 {
            let (norm, phase) = spectrum[i].to_polar();
            let bin_center_freq = TAU * i as f32 / len as f32;

            let phase_diff = phase - prev_input_phases[i] - bin_center_freq * slide_size as f32;
            let phase_diff = wrap_phase(phase_diff);
            prev_input_phases[i] = phase;
            let bin_deviation = phase_diff * len as f32 / slide_size as f32 / TAU;

            pre[i] = [norm, i as f32 + bin_deviation];
        }

        let mut post = vec![[0.0; 2]; len];
        for i in 0..len / 2 + 1 {
            let shifted_bin = (i as f32 / pitch_change_amount).round() as usize;
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
            let bin_deviation = post[i][1] - i as f32;
            let mut phase_diff = bin_deviation * TAU * slide_size as f32 / len as f32;
            let bin_center_freq = TAU * i as f32 / len as f32;
            phase_diff += bin_center_freq * slide_size as f32;

            let phase = wrap_phase(prev_output_phases[i] + phase_diff);
            shifted_spectrum[i] = Complex32::from_polar(post[i][0], phase);
            prev_output_phases[i] = phase;
        }

        fill_right_part_of_spectrum(&mut shifted_spectrum);

        shifted_spectrum
    }
}

pub fn wrap_phase(phase: f32) -> f32 {
    if phase >= 0.0 {
        (phase + PI) % TAU - PI
    } else {
        (phase - PI) % TAU + PI
    }
}

pub fn remove_aliasing<T: Num + Zero + One + Copy>(
    pitch_change_amount: f32,
    fine_structure: &mut [T],
) {
    let len = fine_structure.len();

    if pitch_change_amount < 1.0 {
        let nyquist = (len as f32 / 2.0 * pitch_change_amount).round() as usize;
        fine_structure[nyquist..len / 2].fill(T::zero());

        for i in 1..len / 2 {
            fine_structure[len - i] = fine_structure[i];
        }
    }
}
