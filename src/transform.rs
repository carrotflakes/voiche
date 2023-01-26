use std::iter::Sum;

use rustfft::num_traits;

use crate::float::Float;

pub fn transform<T: Float + Sum>(
    window_size: usize,
    slide_size: usize,
    mut process: impl FnMut(&[T]) -> Vec<T>,
    buf: &[T],
) -> Vec<T> {
    if buf.is_empty() {
        return vec![];
    }

    let mut output = Vec::with_capacity(buf.len());

    for i in 0..(buf.len() - 1) / slide_size + 1 {
        let i = i * slide_size;
        let mut buf = buf[i..(i + window_size).min(buf.len())].to_vec();
        buf.resize(window_size, T::zero());
        buf = process(&buf);
        buffer_overlapping_write(slide_size, &mut output, &buf);
    }
    output
}

pub struct Transformer<T: num_traits::Float + Copy + Sum, F: FnMut(&[T]) -> Vec<T>> {
    window_size: usize,
    input_overlap_size: usize,
    input_buffer: Vec<T>,
    output_buffer: Vec<T>,
    process_fn: F,
}

impl<T: Float + Sum, F: FnMut(&[T]) -> Vec<T>> Transformer<T, F> {
    pub fn new(window_size: usize, slide_size: usize, process_fn: F) -> Self {
        let input_overlap_size = window_size - slide_size;
        Transformer {
            window_size,
            input_overlap_size,
            input_buffer: vec![T::zero(); input_overlap_size],
            output_buffer: vec![],
            process_fn,
        }
    }

    pub fn input_slice(&mut self, slice: &[T]) {
        self.input_buffer.extend(slice.iter().copied());
    }

    pub fn output_slice_exact(&mut self, slice: &mut [T]) -> bool {
        let overlap_size = self.input_overlap_size;
        if self.output_buffer.len() >= slice.len() + overlap_size {
            slice.copy_from_slice(&self.output_buffer[..slice.len()]);
            self.output_buffer.drain(0..slice.len());
            true
        } else {
            false
        }
    }

    pub fn finish(mut self, vec: &mut Vec<T>) {
        self.input_buffer
            .extend(vec![T::zero(); self.window_size - self.input_overlap_size]);
        self.process();
        vec.extend_from_slice(&self.output_buffer);
    }

    pub fn process(&mut self) {
        let Transformer {
            window_size,
            input_overlap_size,
            input_buffer,
            output_buffer,
            process_fn,
        } = self;
        let slide_size = *window_size - *input_overlap_size;

        while input_buffer.len() >= *window_size {
            let buf = process_fn(&input_buffer[..*window_size]);

            buffer_overlapping_write(slide_size, output_buffer, &buf);

            input_buffer.splice(0..slide_size, []);
        }
    }
}

pub fn buffer_overlapping_write<T: Float>(least_size: usize, buffer: &mut Vec<T>, other: &[T]) {
    assert!(least_size <= other.len());

    if buffer.len() < least_size {
        buffer.resize(least_size, T::zero());
    }

    let mut iter = other.iter().copied();
    let len = buffer.len();
    let overlap_size = other.len() - least_size;
    for i in len - overlap_size..len {
        buffer[i] = buffer[i] + iter.next().unwrap();
    }
    buffer.extend(iter);
}

// #[test]
// fn test() {
//     let mut transform = Transformer::new(vec![1.0; 5], 3, |_| {});
//     let mut output = vec![0.0; 8];
//     let mut all_output = vec![];

//     transform.input_slice(&[0.1; 5]);
//     assert_eq!(transform.output_slice_exact(&mut output), false);
//     transform.process();
//     assert_eq!(transform.output_slice_exact(&mut output), false);

//     transform.input_slice(&[0.2; 3]);
//     assert_eq!(transform.output_slice_exact(&mut output), false);
//     transform.process();
//     assert_eq!(transform.output_slice_exact(&mut output), false);

//     transform.input_slice(&[0.4; 4]);
//     assert_eq!(transform.output_slice_exact(&mut output), false);
//     transform.process();
//     assert_eq!(transform.output_slice_exact(&mut output), true);
//     all_output.extend_from_slice(&output);

//     dbg!(&all_output);
//     dbg!(transform.input_buffer.len());
//     dbg!(transform.output_buffer.len());
//     transform.input_slice(&[0.8; 10]);
//     assert_eq!(transform.output_slice_exact(&mut output), false);
//     transform.process();
//     assert_eq!(transform.output_slice_exact(&mut output), true);
//     all_output.extend_from_slice(&output);

//     dbg!(&all_output);
// }

// #[test]
// fn test2() {
//     let mut transform = Transformer::new(vec![1.0; 8], 4, |_| {});
//     let input: Vec<_> = (0..123).map(|x| (x as f32) % 16.0).collect();
//     let mut all_output = vec![];

//     transform.input_slice(&input);
//     transform.process();
//     let mut output = vec![0.0; 7];
//     while transform.output_slice_exact(&mut output) {
//         all_output.extend_from_slice(&output);
//     }
//     transform.finish(&mut all_output);

//     assert_eq!(all_output.len(), input.len());
//     dbg!(&all_output);
// }
