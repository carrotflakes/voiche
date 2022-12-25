use std::iter::Sum;

use rustfft::num_traits;

pub fn transform<T: num_traits::Float + Copy + Sum>(
    slide_size: usize,
    window: Vec<T>,
    mut process: impl FnMut(&[T]) -> Vec<T>,
    buf: &[T],
) -> Vec<T> {
    let mut output = vec![T::zero(); buf.len()];

    if buf.is_empty() {
        return output;
    }

    let output_scale = T::from(slide_size).unwrap() / window.iter().copied().sum::<T>();

    for i in 0..(buf.len() - 1) / slide_size + 1 {
        let i = i * slide_size;
        let mut b: Vec<_> = buf[i..]
            .iter()
            .zip(window.iter())
            .map(|(&x, &y)| x * y)
            .collect();
        b.resize(window.len(), T::zero());
        let b = process(&b);
        let b: Vec<_> = b
            .into_iter()
            .enumerate()
            .map(|(i, x)| x * window[i] * output_scale)
            .collect();
        for (x, &y) in output[i..].iter_mut().zip(b.iter()) {
            *x = *x + y;
        }
    }
    output
}

pub struct Transformer<T: num_traits::Float + Copy + Sum, F: FnMut(&[T]) -> Vec<T>> {
    window: Vec<T>,
    slide_size: usize,
    input_buffer: Vec<T>,
    output_buffer: Vec<T>,
    process_fn: F,
}

impl<T: num_traits::Float + Copy + Sum, F: FnMut(&[T]) -> Vec<T>> Transformer<T, F> {
    pub fn new(window: Vec<T>, slide_size: usize, process_fn: F) -> Self {
        assert!(window.len() >= slide_size);
        let overlap_size = window.len() - slide_size;
        Transformer {
            window,
            slide_size,
            input_buffer: vec![T::zero(); overlap_size],
            output_buffer: vec![T::zero(); overlap_size],
            process_fn,
        }
    }

    fn overlap_size(&self) -> usize {
        self.window.len() - self.slide_size
    }

    pub fn input_slice(&mut self, slice: &[T]) {
        self.input_buffer.extend(slice.iter().copied());
    }

    pub fn output_slice_exact(&mut self, slice: &mut [T]) -> bool {
        let overlap_size = self.overlap_size();
        if self.output_buffer.len() >= slice.len() + overlap_size * 2 {
            slice.copy_from_slice(&self.output_buffer[overlap_size..][..slice.len()]);
            self.output_buffer.splice(0..slice.len(), []);
            true
        } else {
            false
        }
    }

    pub fn finish(mut self, vec: &mut Vec<T>) {
        let last_output_size =
            self.input_buffer.len() + self.output_buffer.len() - self.overlap_size() * 2;
        self.input_buffer.extend(vec![T::zero(); self.slide_size]);
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
        let output_scale = T::from(slide_size).unwrap() / window.iter().copied().sum::<T>();

        while input_buffer.len() >= window_size {
            let b: Vec<_> = input_buffer[..window_size]
                .iter()
                .zip(window.iter())
                .map(|(&x, &y)| x * y)
                .collect();

            let b = process_fn(&b);
            debug_assert_eq!(window_size, b.len());

            let mut iter = b
                .into_iter()
                .zip(window.iter())
                .map(|(x, &y)| x * y * output_scale);
            let output_buffer_len = output_buffer.len();
            for x in output_buffer[output_buffer_len - overlap_size..].iter_mut() {
                *x = *x + iter.next().unwrap();
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
