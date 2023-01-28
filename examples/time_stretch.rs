mod wav;

use voiche::{api, overlapping_flatten::OverlappingFlattenTrait, windows};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;
    let time_rate = 1.1;

    wav::wav_file_convert("ts", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                // resample(buf, 0.7)

                // let window = windows::trapezoid_window(window_size, window_size - slide_size);
                // buf.windows(window_size)
                //     .step_by((slide_size as f32 * time_rate) as usize)
                //     .map(|b| apply_window(&window, b.iter().copied()))
                //     .overlapping_flatten(window_size - slide_size)
                //     .collect::<Vec<_>>()

                resample(&buf, time_rate)
                    .windows(window_size)
                    .step_by(slide_size)
                    .map(api::pitch_shift(
                        windows::hann_window(window_size),
                        windows::trapezoid_window(window_size, window_size - slide_size),
                        slide_size,
                        time_rate.log2(),
                    ))
                    .overlapping_flatten(window_size - slide_size)
                    .collect::<Vec<_>>()

                // buf.windows(window_size)
                // .step_by((slide_size as f32 * time_rate) as usize)
                // .map(pitch_shift(
                //     windows::hann_window(window_size),
                //     windows::trapezoid_window(window_size, window_size - slide_size),
                //     slide_size,
                //     (slide_size as f32 * time_rate) as usize,
                //     time_rate.log2(),
                // ))
                // .overlapping_flatten(window_size - slide_size)
                // .collect::<Vec<_>>()
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

pub fn pitch_shift<T: voiche::float::Float + std::iter::Sum>(
    pre_window: Vec<T>,
    post_window: Vec<T>,
    slide_size: usize,
    slide_size_2: usize,
    pitch: T,
) -> impl FnMut(&[T]) -> Vec<T> {
    assert_eq!(pre_window.len(), post_window.len());

    let window_size = pre_window.len();
    let fft = voiche::fft::Fft::new(window_size);
    let mut pitch_shift = voiche::pitch_shift::pitch_shifter(window_size);

    move |buf| {
        api::retouch_spectrum(
            &fft,
            &pre_window,
            &post_window,
            slide_size,
            &buf,
            |spectrum| {
                voiche::pitch_shift::process_spectrum(
                    slide_size_2,
                    &mut pitch_shift,
                    pitch,
                    spectrum,
                );
            },
        )
    }
}
