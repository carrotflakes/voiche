mod wav;

use voiche::{transform::transform, voice_change::transform_processor, windows::hann_window};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, buf) = wav::load(&file);
    dbg!(wav::power(&buf));

    let start = std::time::Instant::now();
    let window_size = 1024;
    let window = hann_window(window_size);
    let slide_size = window_size / 4;

    let buf = transform(
        slide_size,
        window,
        transform_processor(window_size, slide_size, 20, -0.2, -0.4),
        &buf,
    );
    dbg!(start.elapsed());
    dbg!(wav::power(&buf));

    wav::save(file.replace(".", "_out."), spec, buf);
}
