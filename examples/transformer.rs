use hound::{SampleFormat, WavSpec};
use voiche::{
    transform::{hann_window, Transformer},
    voice_change::{self, power},
};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("epic.wav".to_string());

    let mut reader = hound::WavReader::open(&file).unwrap();
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

    let start = std::time::Instant::now();
    let window_size = 1024;
    let slide_size = window_size / 4;
    let window = hann_window(window_size);
    let mut transformer = Transformer::new(
        window,
        slide_size,
        voice_change::transform_processor(window_size, slide_size, 20, -0.2, -0.4),
    );
    transformer.input_slice(&buf);
    let mut buf = Vec::new();
    transformer.finish(&mut buf);

    dbg!(start.elapsed());
    dbg!(power(&buf));

    let mut writer = hound::WavWriter::create(
        file.replace(".", "_out."),
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
