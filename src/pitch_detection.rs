use rustfft::num_complex::Complex;

use crate::{apply_window, fft::Fft, Float};

/// Detect pitch from a buffer.
/// Returns a tuple of wavelength and gain.
pub fn pitch_detect<T: Float>(
    fft: &Fft<T>,
    window: &[T],
    buf: &[T],
    min_wavelength: T,
    peak_threshold: T,
) -> Option<(T, T)> {
    let buf: Vec<_> = apply_window(window, buf.iter().copied()).collect();
    let nsdf = compute_nsdf(&fft, &buf);
    let mut peaks = compute_peaks(&nsdf[..nsdf.len() / 2]);
    peaks.retain(|p| min_wavelength < p.0);
    let max_peak = peaks.iter().fold(T::zero(), |a, p| a.max(p.1));
    if peak_threshold < max_peak {
        peaks
            .iter()
            .find(|p| max_peak * T::from(0.9).unwrap() <= p.1)
            .cloned()
    } else {
        None
    }
}

/// Normalized Square Difference Function (NSDF)
pub fn compute_nsdf<T: Float>(fft: &Fft<T>, buf: &[T]) -> Vec<T> {
    let mut cmps: Vec<_> = buf.iter().copied().map(Complex::from).collect();
    fft.forward(&mut cmps);
    compute_nsdf_from_spectrum(fft, buf, cmps)
}

pub fn compute_nsdf_from_spectrum<T: Float>(
    fft: &Fft<T>,
    buf: &[T],
    mut spectrum: Vec<Complex<T>>,
) -> Vec<T> {
    for x in &mut spectrum {
        *x = Complex::from(x.norm_sqr());
    }
    fft.inverse(&mut spectrum);

    let len = buf.len();
    let mut nsdf = vec![T::zero(); len];
    let mut m = T::epsilon();
    for i in 0..len {
        let inv = len - i - 1;
        m = m + buf[i].powi(2) + buf[inv].powi(2);
        nsdf[inv] = T::from(2.0).unwrap() * spectrum[inv].re / (m * T::from(len).unwrap());
    }

    nsdf
}

pub fn compute_peaks<T: Float>(nsdf: &[T]) -> Vec<(T, T)> {
    let zero = T::zero();
    let mut peak = (zero, zero);
    let mut peaks = Vec::with_capacity(32);
    let mut is_first = true;

    for i in 0..nsdf.len().saturating_sub(3) {
        if nsdf[i + 1] < T::zero() {
            if zero < peak.1 {
                peaks.push(peak);
                peak = (zero, zero);
            }
            is_first = false;
            continue;
        }

        if !is_first && nsdf[i + 1] - nsdf[i] > zero && nsdf[i + 2] - nsdf[i + 1] <= zero {
            let t = T::from(2.0).unwrap()
                * (nsdf[i] - T::from(2.0).unwrap() * nsdf[i + 1] + nsdf[i + 2]);
            let d = (nsdf[i] - nsdf[i + 2]) / t;
            let c = nsdf[i + 1] - t * d * d / T::from(4.0).unwrap();
            if peak.1 < c {
                peak = (T::from(i).unwrap() + d, c);
            }
        }
    }
    peaks
}
