mod wav;

use rustfft::num_complex::Complex;
use voiche::{fft::Fft, griffin_lim::griffin_lim, transform::transform, windows};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, bufs) = wav::load(&file);
    dbg!(wav::power(&bufs[0]));

    let start = std::time::Instant::now();
    let window_size = 1024;
    let window = windows::hann_window(window_size);
    let slide_size = window_size / 4;

    let fft = Fft::new(window_size);
    let bufs: Vec<_> = bufs
        .iter()
        .map(|buf| {
            transform(
                slide_size,
                window.clone(),
                |buf| {
                    fft.retouch_spectrum(buf, |buf| {
                        let b: Vec<_> = buf.iter().map(|&x| Complex::new(x.re, 0.0)).collect();
                        buf.copy_from_slice(&griffin_lim(&fft, 20, &b));
                    })
                },
                buf,
            )
        })
        .collect();

    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_gl."), spec, bufs);
}
