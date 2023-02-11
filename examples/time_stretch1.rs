mod wav;

use voiche::{apply_window, overlapping_flatten::OverlappingFlattenTrait, windows};

fn main() {
    let window_size = 512;
    let slide_size = window_size / 4;
    let time_rate = 1.1;
    let window = windows::trapezoid_window(window_size, window_size - slide_size);

    wav::wav_file_convert("ts1", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                buf.windows(window_size)
                    .step_by((slide_size as f32 / time_rate) as usize)
                    .map(|b| apply_window(&window, b.iter().copied()))
                    .overlapping_flatten(window_size - slide_size)
                    .collect::<Vec<_>>()
            })
            .collect()
    });
}
