// parec -r --raw --format=s16ne --channels=1 | cargo run --release --example stdinout 2> /dev/null | pacat --raw --format=s16ne --channels=1

use std::{
    convert::TryInto,
    io::{Read, Write},
};

use voiche::{transform::Transformer, voice_change, windows};

fn main() {
    let window_size = 1024;
    let window = windows::hann_window(window_size);
    let slide_size = window_size / 4;
    let envelope_order = 20;
    let formant = 0.2;
    let pitch = 0.4;

    let mut transformer = Transformer::new(
        window.clone(),
        slide_size,
        voice_change::transform_processor(window_size, slide_size, envelope_order, formant, pitch),
    );

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
