mod wav;

use voiche::{pitch_shift, transform::transform, windows::hann_window};

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
        slide_size,
        window.clone(),
        pitch_shift::transform_processor(window_size, slide_size, -0.4),
        &buf,
    )}).collect();
    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_out."), spec, bufs);
}
