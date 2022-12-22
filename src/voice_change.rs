use std::f32::consts::{PI, TAU};

use crate::{
    fft::{fill_right_part_of_spectrum, fix_scale, Fft},
    transform::{hann_window, transform},
};
use rustfft::{num_complex::Complex32, num_traits::Zero};

pub fn voice_change(envelope_order: usize, formant: f32, pitch: f32, buf: &[f32]) -> Vec<f32> {
    assert!(0 < envelope_order && envelope_order < buf.len() / 2);
    let window_size = 1024;
    let window = hann_window(window_size);
    let slide_size = window_size / 4;
    let fft = Fft::new(window_size);
    let mut pitch_shift = pitch_shifter(window_size);

    transform(slide_size, window, buf, |buf| {
        fft.retouch_spectrum(buf, |spectrum| {
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
        })
    })
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
        for i in 1..len / 2 {
            shifted_spectrum[len - i] = shifted_spectrum[i].conj();
        }
        shifted_spectrum
    }
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

pub fn remove_aliasing(pitch_change_amount: f32, fine_structure: &mut [f32]) {
    let len = fine_structure.len();

    if pitch_change_amount < 1.0 {
        let nyquist = (len as f32 / 2.0 * pitch_change_amount).round() as usize;
        fine_structure[nyquist..len / 2].fill(0.0);

        for i in 1..len / 2 {
            fine_structure[len - i] = fine_structure[i];
        }
    }
}

pub fn wrap_phase(phase: f32) -> f32 {
    if phase >= 0.0 {
        (phase + PI) % TAU - PI
    } else {
        (phase - PI) % TAU + PI
    }
}

pub fn power(buf: &[f32]) -> f32 {
    (buf.iter().map(|&x| x.powi(2)).sum::<f32>() / buf.len() as f32).sqrt()
}
