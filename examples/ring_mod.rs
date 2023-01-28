mod wav;

fn main() {
    wav::wav_file_convert("rm", |_sample_rate, channels| {
        channels
            .into_iter()
            .map(|buf| {
                buf.iter()
                    .enumerate()
                    .map(|(i, &x)| x * (i as f32 * 0.04).sin())
                    .collect()
            })
            .collect()
    });
}
