mod wav;

use voiche::{api, transform, windows};

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
    let envelope_order = window_size / 8;
    let formant = -0.2;
    let pitch = -0.4;

    let bufs: Vec<_> = bufs
        .iter()
        .map(|buf| {
            let process = api::voice_change(
                windows::hann_window(window_size),
                windows::trapezoid_window(window_size, window_size - slide_size),
                slide_size,
                envelope_order,
                formant,
                pitch,
            );

            transform::transform(window_size, slide_size, process, &buf)
        })
        .collect();

    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_main."), spec, bufs);
}
