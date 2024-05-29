/// Time stretching by repeating or deleting between zero-crossing points.
mod wav;

fn main() {
    let time_rate = 1.5;

    wav::wav_file_convert("ts3", |sample_rate, channels| {
        let min_interval = sample_rate as usize / 100;
        let overlap_size = min_interval / 2;

        channels
            .into_iter()
            .map(|buf| {
                let mut output = Vec::new();

                let mut i = 0;
                let mut zcs = zero_crosses(&buf, min_interval);
                let mut zc = zcs.next().unwrap_or(buf.len());
                while i < buf.len() {
                    let desired_len = (i as f32 * time_rate) as usize;
                    while desired_len > output.len() {
                        // output.extend(buf[i..zc].iter().copied());
                        buffer_cross_fading_write(
                            overlap_size,
                            &mut output,
                            buf[i.saturating_sub(overlap_size)..zc].iter().copied(),
                        );
                    }
                    i = zc;
                    zc = zcs.next().unwrap_or(buf.len());
                }

                output.resize((buf.len() as f32 * time_rate) as usize, 0.0);
                output
            })
            .collect()
    });
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

pub fn buffer_cross_fading_write<T: voiche::float::Float>(
    overlap_size: usize,
    buffer: &mut Vec<T>,
    mut other: impl Iterator<Item = T>,
) {
    let len = buffer.len();
    let overlap_size = overlap_size.min(len);
    for i in 0..overlap_size {
        let Some(x) = other.next() else {
            return;
        };
        let j = len - overlap_size + i;
        let r = T::from(i).unwrap() / T::from(overlap_size).unwrap();
        buffer[j] = buffer[j].clone() + (x - buffer[j].clone()) * r;
    }
    buffer.extend(other);
}
