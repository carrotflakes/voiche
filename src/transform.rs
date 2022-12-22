pub fn transform(
    slide_size: usize,
    window: Vec<f32>,
    buf: &[f32],
    mut process: impl FnMut(&[f32]) -> Vec<f32>,
) -> Vec<f32> {
    let mut output = vec![0.0; buf.len()];
    let output_scale = slide_size as f32 / window.iter().sum::<f32>();

    for i in 0..buf.len() / slide_size {
        let i = i * slide_size;
        let mut b: Vec<_> = buf[i..]
            .iter()
            .zip(window.iter())
            .map(|(x, y)| x * y)
            .collect();
        b.resize(window.len(), 0.0);
        let b = process(&b);
        let b: Vec<_> = b
            .into_iter()
            .enumerate()
            .map(|(i, x)| x * window[i] * output_scale)
            .collect();
        for (x, y) in output[i..].iter_mut().zip(b.iter()) {
            *x += y;
        }
    }
    output
}

pub fn process_nop(buf: &[f32]) -> Vec<f32> {
    buf.to_vec()
}

pub fn process_rev(buf: &[f32]) -> Vec<f32> {
    let mut buf = buf.to_vec();
    buf.reverse();
    buf
}

pub fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| 0.5 * (1.0 - (i as f32 * std::f32::consts::TAU / size as f32).cos()))
        .collect()
}
