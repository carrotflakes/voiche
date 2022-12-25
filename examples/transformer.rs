mod wav;

use voiche::{power, transform::Transformer, voice_change, windows::hann_window};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, buf) = wav::load(&file);
    dbg!(power(&buf));

    let start = std::time::Instant::now();
    let window_size = 1024;
    let slide_size = window_size / 4;
    let window = hann_window(window_size);
    let mut transformer = Transformer::new(
        window,
        slide_size,
        voice_change::transform_processor(window_size, slide_size, 20, -0.2, -0.4),
    );
    transformer.input_slice(&buf);
    let mut buf = Vec::new();
    transformer.finish(&mut buf);

    dbg!(start.elapsed());
    dbg!(power(&buf));

    wav::save(file.replace(".", "_out."), spec, buf);
}
