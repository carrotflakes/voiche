mod wav;

use voiche::{api, overlapping_flatten::OverlappingFlattenTrait, windows};

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

pub fn resample<T: voiche::float::Float>(buf: &[T], rate: T) -> Vec<T> {
    let mut output = vec![
        T::zero();
        (T::from(buf.len()).unwrap() * rate)
            .ceil()
            .to_usize()
            .unwrap()
    ];

    for i in 0..output.len() {
        let p = T::from(i).unwrap() / rate;
        let j = p.to_usize().unwrap();

        let x = buf[j];
        let y = buf.get(j + 1).copied().unwrap_or(T::zero());
        let z = buf.get(j + 2).copied().unwrap_or(T::zero());

        output[i] = y - (x - z) * p.fract() / T::from(4.0).unwrap();
    }

    output
}
