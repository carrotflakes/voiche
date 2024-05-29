/// Time stretch through resampling and pitch shifting.
mod wav;

use voiche::{api, overlapping_flatten::OverlappingFlattenTrait, resample, windows};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;
    let time_rate = 1.1;

    wav::wav_file_convert("ts2", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                resample(&buf, time_rate)
                    .windows(window_size)
                    .step_by(slide_size)
                    .map(api::pitch_shift(
                        windows::hann_window(window_size),
                        windows::trapezoid_window(window_size, window_size - slide_size),
                        slide_size,
                        time_rate,
                    ))
                    .overlapping_flatten(window_size - slide_size)
                    .collect::<Vec<_>>()
            })
            .collect()
    });
}
