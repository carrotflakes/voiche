mod wav;

use voiche::{api, transform::Transformer, windows};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;
    let envelope_order = window_size / 8;
    let formant = -0.2;
    let pitch = -0.4;

    wav::wav_file_convert("tr", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                let mut transformer = Transformer::new(
                    window_size,
                    slide_size,
                    api::voice_change(
                        windows::hann_window(window_size),
                        windows::trapezoid_window(window_size, window_size - slide_size),
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
            .collect()
    });
}
