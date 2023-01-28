mod wav;

use voiche::{api, transform::transform, windows};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;
    let pitch_fn = |freq: f32| {
        let nn = (freq / 440.0).log2() * 12.0;
        // approximately scale
        let nn_correct = ((nn * (7.0 / 12.0)).round() / (7.0 / 12.0)).round();
        -(nn - nn_correct) / 12.0
    };
    // let pitch_fn = |freq: f32| {
    //     -(freq / 220.0).log2()
    // };

    wav::wav_file_convert("pc", |sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                let process = api::pitch_correct(
                    windows::hann_window(window_size),
                    windows::trapezoid_window(window_size, window_size - slide_size),
                    slide_size,
                    sample_rate,
                    pitch_fn,
                );

                transform(window_size, slide_size, process, &buf)
            })
            .collect()
    });
}
