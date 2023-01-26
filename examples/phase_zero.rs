mod wav;

use rustfft::num_complex::Complex;
use voiche::{api, fft::Fft, transform::transform, windows};

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
                window_size,
                slide_size,
                |buf| {
                    api::retouch_spectrum(&fft, &window, &window, slide_size, buf, |buf| {
                        buf.iter_mut()
                            .for_each(|c| *c = Complex::from_polar(c.norm(), 0.0))
                    })
                },
                buf,
            )
        })
        .collect();

    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_pz."), spec, bufs);
}
