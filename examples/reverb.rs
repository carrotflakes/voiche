mod wav;

use std::iter::Sum;

use voiche::{
    api, apply_window, fft::Fft, num_complex::Complex, num_traits::Zero, transform, windows, Float,
};

fn main() {
    let window_size = 1024 * 4;
    let slide_size = window_size / 8;
    let pre_window = windows::hann_window(window_size);

    let scale = 0.2;

    let fft = Fft::new(window_size);
    let (_, ir_buf) = wav::load("./ir.wav");
    let ir_specs: Vec<_> = ir_buf[0]
        .windows(window_size)
        .step_by(slide_size)
        .map(|buf| {
            let mut spec: Vec<_> = apply_window(&pre_window, buf.iter().copied())
                .map(|x| x * scale)
                .map(Complex::from)
                .collect();
            fft.forward(&mut spec);
            spec
        })
        .collect();
    dbg!(ir_specs.len());

    wav::wav_file_convert("rv", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                let process = reverb(
                    pre_window.clone(),
                    windows::trapezoid_window(window_size, window_size - slide_size),
                    slide_size,
                    ir_specs.clone(),
                );

                transform::transform(window_size, slide_size, process, &buf)
            })
            .collect()
    });
}

pub fn reverb<T: Float + Sum + Default>(
    pre_window: Vec<T>,
    post_window: Vec<T>,
    slide_size: usize,
    ir_specs: Vec<Vec<Complex<T>>>,
) -> impl FnMut(&[T]) -> Vec<T> {
    assert_eq!(pre_window.len(), post_window.len());

    let window_size = pre_window.len();
    let fft = Fft::new(window_size);
    let mut specs: Vec<Vec<Complex<T>>> = vec![];

    move |buf| {
        api::retouch_spectrum(
            &fft,
            &pre_window,
            &post_window,
            slide_size,
            &buf,
            |spectrum| {
                specs.insert(0, spectrum.to_vec());
                specs.truncate(ir_specs.len());

                spectrum.fill(Complex::zero()); // Comment out to add the dry.

                // Add the wet.
                for (spec, ir_spec) in specs.iter().zip(ir_specs.iter()) {
                    for i in 0..window_size {
                        spectrum[i] = spectrum[i] + spec[i] * ir_spec[i];
                    }
                }
            },
        )
    }
}
