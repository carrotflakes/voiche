mod wav;

fn main() {
    wav::wav_file_convert("samy", |sample_rate, channels| {
        let min_interval = sample_rate as usize / 10000;

        channels
            .into_iter()
            .map(|buf| {
                let mut output = Vec::new();

                let mut zcs = zero_crosses(&buf, min_interval);
                let mut zc = zcs.next().unwrap_or(buf.len());
                let mut range = 0..zc;
                while output.len() < buf.len() {
                    // output.extend(buf[range.clone()].iter().map(|x| (x * 16.0).round() / 16.0));
                    // output.extend(buf[range.clone()].iter().copied());
                    output.extend(buf[range.clone()].iter().rev().copied());
                    while zc < output.len().min(buf.len()) {
                        let next_zc = zcs.next().unwrap_or(buf.len());
                        if !similar(&buf[range.clone()], &buf[zc..next_zc]) {
                            range = zc..next_zc;
                        }
                        zc = next_zc;
                    }
                }
                output.resize(buf.len(), 0.0);
                output
            })
            .collect()
    });
}

fn similar<T: voiche::float::Float>(a: &[T], b: &[T]) -> bool {
    let time_tolerance = 1.2;
    let rms_tolerance = 0.5;

    if a.len() as f32 / b.len() as f32 > time_tolerance
        || b.len() as f32 / a.len() as f32 > time_tolerance
    {
        return false;
    }

    let rms = (a
        .iter()
        .zip(b)
        .map(|(a, b)| (*a - *b).powi(2))
        .reduce(|a, b| a + b)
        .unwrap_or(T::zero())
        / T::from_usize(a.len().max(b.len())).unwrap())
    .sqrt();
    rms < T::from_f32(rms_tolerance).unwrap()
}

pub fn zero_crosses<T: voiche::float::Float>(
    buf: &[T],
    min_interval: usize,
) -> impl Iterator<Item = usize> + '_ {
    let mut prev = 0;
    buf.windows(2).enumerate().filter_map(move |(i, w)| {
        if i - prev >= min_interval && w[0] <= T::zero() && w[1] >= T::zero() {
            prev = i;
            Some(i)
        } else {
            None
        }
    })
}
