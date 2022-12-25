mod wav;

use voiche::{pitch_shift, power, transform::transform, windows::hann_window};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, buf) = wav::load(&file);
    dbg!(power(&buf));

    // let buf = vc(&buf, process_nop);
    let start = std::time::Instant::now();
    let window_size = 1024;
    let window = hann_window(window_size);
    let slide_size = window_size / 4;

    let buf = transform(
        slide_size,
        window,
        pitch_shift::transform_processor(window_size, slide_size, -0.4),
        &buf,
    );
    dbg!(start.elapsed());
    dbg!(power(&buf));

    wav::save(file.replace(".", "_out."), spec, buf);
}
