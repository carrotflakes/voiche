use std::path::Path;

use hound::{SampleFormat, WavSpec};

pub fn load(p: impl AsRef<Path>) -> (WavSpec, Vec<Vec<f32>>) {
    let mut reader = hound::WavReader::open(&p).unwrap();
    let spec = reader.spec();
    dbg!(&spec);
    let mut bufs: Vec<_> = (0..spec.channels).map(|_| vec![]).collect();
    match &spec {
        WavSpec {
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
            ..
        } => {
            for (i, x) in reader.samples::<f32>().map(|x| x.unwrap()).enumerate() {
                bufs[i % spec.channels as usize].push(x);
            }
        }
        WavSpec {
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
            ..
        } => {
            for (i, x) in reader
                .samples::<i16>()
                .map(|x| x.unwrap() as f32 / i16::MAX as f32)
                .enumerate()
            {
                bufs[i % spec.channels as usize].push(x);
            }
        }
        _ => panic!("unexpected wav format"),
    }
    (spec, bufs)
}

pub fn save(p: impl AsRef<Path>, spec: WavSpec, bufs: Vec<Vec<f32>>) {
    let mut writer = hound::WavWriter::create(p, spec).unwrap();
    match &spec {
        WavSpec {
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
            ..
        } => {
            for i in 0..bufs[0].len() {
                for c in 0..spec.channels as usize {
                    writer.write_sample(bufs[c][i]).unwrap();
                }
            }
        }
        WavSpec {
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
            ..
        } => {
            for i in 0..bufs[0].len() {
                for c in 0..spec.channels as usize {
                    writer
                        .write_sample(
                            (bufs[c][i] * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32)
                                as i16,
                        )
                        .unwrap()
                }
            }
        }
        _ => panic!("unexpected wav format"),
    }
    writer.finalize().unwrap();
}

pub fn power<T: voiche::float::Float + std::iter::Sum>(buf: &[T]) -> T {
    (buf.iter().map(|&x| x.powi(2)).sum::<T>() / T::from(buf.len()).unwrap()).sqrt()
}

#[allow(dead_code)]
fn main() {}
