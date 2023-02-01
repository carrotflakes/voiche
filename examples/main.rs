mod wav;

use voiche::{api, transform, windows};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;
    let envelope_order = window_size / 8;
    let formant = (-0.2f32).exp2();
    let pitch = (-0.4f32).exp2();

    wav::wav_file_convert("main", |_sample_rate, channels| {
        channels
            .into_iter()
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
            .collect()
    });
}
