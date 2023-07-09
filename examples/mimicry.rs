// Mimicry demo
// This demo detects voice and mimic it.
// Usage:
// parec -r --raw --format=s16ne --rate 48000 --channels=1 | cargo run --release --example mimicry 2> /dev/null | pacat --raw --format=s16ne --rate 48000 --channels=1

use std::convert::TryInto;

use voiche::{
    api,
    transform::{self, Transformer},
    windows,
};

const SAMPLE_RATE: usize = 48000;
const THRESHOLD: f32 = 0.05;

fn main() {
    let window_size = 1024;
    let slide_size = window_size / 4;
    let pitch = 1.5;

    let mut processor = MimicryProcessor::new(
        SAMPLE_RATE as f32,
        Box::new(move |buf: &[f32]| {
            let process = api::pitch_shift(
                windows::hann_window(window_size),
                windows::trapezoid_window(window_size, window_size - slide_size),
                slide_size,
                pitch,
            );

            transform::transform(window_size, slide_size, process, &buf)
        }),
    );
    transform_mic_to_speaker(window_size, slide_size, move |buf| processor.process(buf));
}

pub struct MimicryProcessor {
    sample_rate: f32,
    mode: Mode,
    buf: Vec<f32>,
    no_voice_time: f32,
    process: Box<dyn FnMut(&[f32]) -> Vec<f32>>,
}

enum Mode {
    Wait,
    Record,
    Speak,
}

impl MimicryProcessor {
    pub fn new(sample_rate: f32, process: Box<dyn FnMut(&[f32]) -> Vec<f32>>) -> Self {
        Self {
            sample_rate,
            mode: Mode::Wait,
            buf: Vec::new(),
            no_voice_time: 0.0,
            process,
        }
    }

    pub fn process(&mut self, buf: &[f32]) -> Vec<f32> {
        let chunk_size = (self.sample_rate * 0.1) as usize;
        match self.mode {
            Mode::Wait => {
                self.buf.extend(buf);
                if chunk_size <= self.buf.len() {
                    self.buf.drain(..self.buf.len() - chunk_size);
                    if detect_voice(buf) {
                        self.mode = Mode::Record;
                        self.no_voice_time = 0.0;
                    }
                }
                vec![0.0; buf.len()]
            }
            Mode::Record => {
                self.buf.extend(buf);
                if detect_voice(&self.buf[self.buf.len() - chunk_size..]) {
                    self.no_voice_time = 0.0;
                } else {
                    self.no_voice_time += buf.len() as f32 / self.sample_rate;
                    if 0.5 <= self.no_voice_time {
                        self.mode = Mode::Speak;
                        self.buf = (self.process)(&self.buf);
                    }
                }
                if self.buf.len() > (self.sample_rate * 10.0) as usize {
                    self.mode = Mode::Speak;
                    self.buf = (self.process)(&self.buf);
                }
                vec![0.0; buf.len()]
            }
            Mode::Speak => {
                let mut out: Vec<_> = self.buf.drain(..buf.len().min(self.buf.len())).collect();
                out.resize(buf.len(), 0.0);
                if self.buf.is_empty() {
                    self.mode = Mode::Wait;
                }
                out
            }
        }
    }
}

fn detect_voice(buf: &[f32]) -> bool {
    let rms = root_mean_square(buf);
    rms > THRESHOLD
}

fn root_mean_square(buf: &[f32]) -> f32 {
    (buf.iter().map(|&x| x * x).sum::<f32>() / buf.len() as f32).sqrt()
}

#[allow(dead_code)]
fn count_zero_cross(buf: &[f32]) -> usize {
    let mut last = buf[0];
    let mut count = 0;
    for &x in buf {
        if x > 0.0 && last <= 0.0 {
            count += 1;
        }
        last = x;
    }
    count
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
