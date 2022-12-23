pub fn transform(
    slide_size: usize,
    window: Vec<f32>,
    buf: &[f32],
    mut process: impl FnMut(&[f32]) -> Vec<f32>,
) -> Vec<f32> {
    let mut output = vec![0.0; buf.len()];

    if buf.is_empty() {
        return output;
    }

    let output_scale = slide_size as f32 / window.iter().sum::<f32>();

    for i in 0..(buf.len() - 1) / slide_size + 1 {
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

pub struct Transformer<T: FnMut(&[f32]) -> Vec<f32>> {
    window: Vec<f32>,
    slide_size: usize,
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    process_fn: T,
}

impl<T: FnMut(&[f32]) -> Vec<f32>> Transformer<T> {
    pub fn new(window: Vec<f32>, slide_size: usize, process_fn: T) -> Self {
        assert!(window.len() >= slide_size);
        let overlap_size = window.len() - slide_size;
        Transformer {
            window,
            slide_size,
            input_buffer: vec![0.0; overlap_size],
            output_buffer: vec![0.0; overlap_size],
            process_fn,
        }
    }

    fn overlap_size(&self) -> usize {
        self.window.len() - self.slide_size
    }

    pub fn input_slice(&mut self, slice: &[f32]) {
        self.input_buffer.extend(slice.iter().copied());
    }

    pub fn output_slice_exact(&mut self, slice: &mut [f32]) -> bool {
        let overlap_size = self.overlap_size();
        if self.output_buffer.len() >= slice.len() + overlap_size * 2 {
            slice.copy_from_slice(&self.output_buffer[overlap_size..][..slice.len()]);
            self.output_buffer.splice(0..slice.len(), []);
            true
        } else {
            false
        }
    }

    pub fn finish(mut self, vec: &mut Vec<f32>) {
        let last_output_size =
            self.input_buffer.len() + self.output_buffer.len() - self.overlap_size() * 2;
        self.input_buffer.extend(vec![0.0; self.slide_size]);
        self.process();
        vec.extend_from_slice(&self.output_buffer[self.overlap_size()..][..last_output_size]);
    }

    pub fn process(&mut self) {
        let overlap_size = self.overlap_size();
        let Transformer {
            window,
            slide_size,
            input_buffer,
            output_buffer,
            process_fn,
        } = self;
        let window_size = window.len();
        let slide_size = *slide_size;
        let output_scale = slide_size as f32 / window.iter().sum::<f32>();

        while input_buffer.len() >= window_size {
            let b: Vec<_> = input_buffer[..window_size]
                .iter()
                .zip(window.iter())
                .map(|(x, y)| x * y)
                .collect();

            let b = process_fn(&b);
            debug_assert_eq!(window_size, b.len());

            let mut iter = b
                .into_iter()
                .zip(window.iter())
                .map(|(x, y)| x * y * output_scale);
            let output_buffer_len = output_buffer.len();
            for x in output_buffer[output_buffer_len - overlap_size..].iter_mut() {
                *x += iter.next().unwrap();
            }
            output_buffer.extend(iter);

            input_buffer.splice(0..slide_size, []);
        }
    }
}

#[test]
fn test() {
    let mut transform = Transformer::new(vec![1.0; 5], 3, |x| x.to_vec());
    let mut output = vec![0.0; 8];
    let mut all_output = vec![];

    transform.input_slice(&[0.1; 5]);
    assert_eq!(transform.output_slice_exact(&mut output), false);
    transform.process();
    assert_eq!(transform.output_slice_exact(&mut output), false);

    transform.input_slice(&[0.2; 3]);
    assert_eq!(transform.output_slice_exact(&mut output), false);
    transform.process();
    assert_eq!(transform.output_slice_exact(&mut output), false);

    transform.input_slice(&[0.4; 4]);
    assert_eq!(transform.output_slice_exact(&mut output), false);
    transform.process();
    assert_eq!(transform.output_slice_exact(&mut output), true);
    all_output.extend_from_slice(&output);

    dbg!(&all_output);
    dbg!(transform.input_buffer.len());
    dbg!(transform.output_buffer.len());
    transform.input_slice(&[0.8; 10]);
    assert_eq!(transform.output_slice_exact(&mut output), false);
    transform.process();
    assert_eq!(transform.output_slice_exact(&mut output), true);
    all_output.extend_from_slice(&output);

    dbg!(&all_output);
}

#[test]
fn test2() {
    let mut transform = Transformer::new(vec![1.0; 8], 4, |x| x.to_vec());
    let input: Vec<_> = (0..123).map(|x| (x as f32) % 16.0).collect();
    let mut all_output = vec![];

    transform.input_slice(&input);
    transform.process();
    let mut output = vec![0.0; 7];
    while transform.output_slice_exact(&mut output) {
        all_output.extend_from_slice(&output);
    }
    transform.finish(&mut all_output);

    assert_eq!(all_output.len(), input.len());
    dbg!(&all_output);
}
