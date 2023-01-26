mod wav;

use voiche::{api, transform::transform, windows::hann_window};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, bufs) = wav::load(&file);
    dbg!(wav::power(&bufs[0]));

    let start = std::time::Instant::now();
    let window_size = 1024;
    let window = hann_window(window_size);
    let slide_size = window_size / 4;

    let bufs: Vec<_> = bufs
        .iter()
        .map(|buf| {
            transform(
                window_size,
                slide_size,
                api::pitch_shift(window.clone(), window.clone(), slide_size, -0.4),
                &buf,
            )
        })
        .collect();
    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_ps."), spec, bufs);
}
