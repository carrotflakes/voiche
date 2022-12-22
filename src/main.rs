use hound::{SampleFormat, WavSpec};
use voice_changer::voice_change::{power, voice_change};

fn main() {
    let p = "karplus-strong.wav";
    let p = "fes.wav";
    let p = "nanachi.wav";
    let mut reader = hound::WavReader::open(p).unwrap();
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
    dbg!(power(&buf));

    // let buf = vc(&buf, process_nop);
    let start = std::time::Instant::now();
    let buf = voice_change(20, -0.2, -0.4, &buf);
    dbg!(start.elapsed());
    dbg!(power(&buf));

    let mut writer = hound::WavWriter::create(
        "output.wav",
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
