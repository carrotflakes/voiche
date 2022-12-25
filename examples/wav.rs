use std::path::Path;

use hound::{SampleFormat, WavSpec};

pub fn load(p: impl AsRef<Path>) -> (WavSpec, Vec<f32>) {
    let mut reader = hound::WavReader::open(&p).unwrap();
    let spec = reader.spec();
    dbg!(&spec);
    let buf: Vec<_> = match &spec {
        WavSpec {
            channels: 2,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
            ..
        } => reader
            .samples::<f32>()
            .map(|x| x.unwrap())
            .enumerate()
            .filter_map(|(i, x)| (i % 2 == 0).then_some(x))
            .collect(),
        WavSpec {
            channels: 2,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
            ..
        } => {
            let buf: Vec<_> = reader
                .samples::<i16>()
                .map(|x| x.unwrap())
                .enumerate()
                .filter_map(|(i, x)| (i % 2 == 0).then_some(x))
                .collect();
            buf.iter().map(|&x| x as f32 / i16::MAX as f32).collect()
        }
        _ => panic!("unexpected wav format"),
    };
    (spec, buf)
}

pub fn save(p: impl AsRef<Path>, spec: WavSpec, buf: Vec<f32>) {
    let mut writer = hound::WavWriter::create(
        p,
        hound::WavSpec {
            channels: 1,
            ..spec
        },
    )
    .unwrap();
    match &spec {
        WavSpec {
            channels: 2,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
            ..
        } => {
            for &x in buf.iter() {
                writer.write_sample(x).unwrap();
            }
        }
        WavSpec {
            channels: 2,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
            ..
        } => {
            for &x in buf.iter() {
                writer
                    .write_sample(
                        (x * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
                    )
                    .unwrap()
            }
        }
        _ => panic!("unexpected wav format"),
    }
    writer.finalize().unwrap();
}

#[allow(dead_code)]
fn main() {}
