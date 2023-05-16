// Voice change mic to speaker
// Usage:
// parec -r --raw --format=s16ne --channels=1 | cargo run --release --example stdinout 2> /dev/null | pacat --raw --format=s16ne --channels=1

use std::convert::TryInto;

use voiche::{api, transform::Transformer, windows};

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;
    let envelope_order = window_size / 8;
    let formant = (-0.2f32).exp2();
    let pitch = (-0.4f32).exp2();

    let process = api::voice_change(
        windows::hann_window(window_size),
        windows::trapezoid_window(window_size, window_size - slide_size),
        slide_size,
        envelope_order,
        formant,
        pitch,
    );

    transform_mic_to_speaker(window_size, slide_size, process);
}

pub fn transform_mic_to_speaker(
    window_size: usize,
    slide_size: usize,
    process: impl FnMut(&[f32]) -> Vec<f32>,
) {
    use std::io::{Read, Write};

    let mut transformer = Transformer::new(window_size, slide_size, process);

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut buf = vec![0u8; 2 * 1024];
    loop {
        let size = stdin.read(&mut buf).unwrap();
        let buf: Vec<_> = buf[..size]
            .chunks(2)
            .map(|c| i16::from_ne_bytes(c.try_into().unwrap()))
            .map(|x| x as f32 / i16::MAX as f32)
            .collect();

        transformer.input_slice(&buf);
        transformer.process();

        let mut buf = vec![0.0; 256];
        while transformer.output_slice_exact(&mut buf) {
            let buf: Vec<_> = buf
                .iter()
                .map(|&x| (x * i16::MAX as f32).round() as i16)
                .flat_map(|x| x.to_ne_bytes())
                .collect();

            stdout.write(&buf).unwrap();
            stdout.flush().unwrap();
        }
    }
}
