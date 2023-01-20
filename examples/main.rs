mod wav;

use voiche::{transform::transform, voice_change::transform_processor, windows};

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
    let envelope_order = window_size / 8;
    let formant = -0.2;
    let pitch = -0.4;

    let bufs: Vec<_> = bufs
        .iter()
        .map(|buf| {
            transform(
                slide_size,
                window.clone(),
                transform_processor(window_size, slide_size, envelope_order, formant, pitch),
                buf,
            )
        })
        .collect();

    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_out."), spec, bufs);
}
