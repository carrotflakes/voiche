mod wav;

use rustfft::num_complex::Complex;
use voiche::{api, fft::Fft, transform, windows};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;
    let fft = Fft::new(window_size);
    let pre_window = windows::hann_window(window_size);
    let post_window = windows::trapezoid_window(window_size, window_size - slide_size);

    wav::wav_file_convert("pz", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                let process = |buf: &[f32]| {
                    api::retouch_spectrum(&fft, &pre_window, &post_window, slide_size, buf, |buf| {
                        for c in buf {
                            *c = Complex::from_polar(c.norm(), 0.0);
                        }
                    })
                };

                transform::transform(window_size, slide_size, process, &buf)
            })
            .collect()
    });
}
