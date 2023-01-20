mod wav;

use voiche::{transform::Transformer, voice_change, windows::hann_window};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, bufs) = wav::load(&file);
    dbg!(wav::power(&bufs[0]));

    let start = std::time::Instant::now();
    let window_size = 1024;
    let slide_size = window_size / 4;
    let window = hann_window(window_size);
    let envelope_order = window_size / 8;
    let formant = -0.2;
    let pitch = -0.4;

    let bufs: Vec<_> = bufs
        .iter()
        .map(|buf| {
            let mut transformer = Transformer::new(
                window.clone(),
                slide_size,
                voice_change::transform_processor(
                    window_size,
                    slide_size,
                    envelope_order,
                    formant,
                    pitch,
                ),
            );
            transformer.input_slice(&buf);
            let mut buf = Vec::new();
            transformer.finish(&mut buf);
            buf
        })
        .collect();

    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_out."), spec, bufs);
}
