mod wav;

use voiche::{api, transform::transform, windows};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let (spec, bufs) = wav::load(&file);
    let sample_rate = spec.sample_rate;
    dbg!(wav::power(&bufs[0]));

    let start = std::time::Instant::now();
    let window_size = 1024;
    let slide_size = window_size / 4;

    let bufs: Vec<_> = bufs
        .iter()
        .map(|buf| {
            let process = api::pitch_correct(
                windows::hann_window(window_size),
                windows::trapezoid_window(window_size, window_size - slide_size),
                slide_size,
                sample_rate,
                |freq: f32| {
                    let nn = (freq / 440.0).log2() * 12.0;
                    // approximately scale
                    let nn_correct = ((nn * (7.0 / 12.0)).round() / (7.0 / 12.0)).round();
                    -(nn - nn_correct) / 12.0
                    // -(freq / 220.0).log2()
                },
            );
            transform(window_size, slide_size, process, buf)
        })
        .collect();

    dbg!(start.elapsed());
    dbg!(wav::power(&bufs[0]));

    wav::save(file.replace(".", "_pc."), spec, bufs);
}
