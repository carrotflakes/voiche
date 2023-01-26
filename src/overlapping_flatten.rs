use std::{iter::Fuse, ops::Add};

pub trait OverlappingFlattenTrait: Iterator + Sized
where
    <Self as Iterator>::Item: IntoIterator,
    <<Self as Iterator>::Item as IntoIterator>::Item: Default + Clone,
{
    fn overlapping_flatten(self, overlap_size: usize) -> OverlappingFlatten<Self> {
        OverlappingFlatten {
            inner: OverlappingFlattenCompat::new(self, overlap_size),
        }
    }
}

impl<T: Iterator + Sized> OverlappingFlattenTrait for T
where
    <Self as Iterator>::Item: IntoIterator,
    <<Self as Iterator>::Item as IntoIterator>::Item: Default + Clone,
{
}

pub struct OverlappingFlatten<I>
where
    I: Iterator,
    <I as Iterator>::Item: IntoIterator,
{
    inner: OverlappingFlattenCompat<I, <I::Item as IntoIterator>::Item>,
}

impl<I> Iterator for OverlappingFlatten<I>
where
    I: Iterator,
    <I as Iterator>::Item: IntoIterator,
    <<I as Iterator>::Item as IntoIterator>::Item: Add<
            <<I as Iterator>::Item as IntoIterator>::Item,
            Output = <<I as Iterator>::Item as IntoIterator>::Item,
        > + Clone,
{
    type Item = <I::Item as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

struct OverlappingFlattenCompat<I, U>
where
    I: Iterator,
    <I as Iterator>::Item: IntoIterator<Item = U>,
{
    iter: Fuse<I>,
    overlap_size: usize,
    buffer: Vec<U>,
    offset: usize,
}

impl<I, U> OverlappingFlattenCompat<I, U>
where
    I: Iterator,
    <I as Iterator>::Item: IntoIterator<Item = U>,
    U: Default + Clone,
{
    fn new(iter: I, overlap_size: usize) -> Self {
        Self {
            iter: iter.fuse(),
            overlap_size,
            buffer: vec![U::default(); overlap_size],
            offset: 0,
        }
    }
}

impl<I, U> Iterator for OverlappingFlattenCompat<I, U>
where
    I: Iterator,
    <I as Iterator>::Item: IntoIterator<Item = U>,
    U: Add<U, Output = U> + Clone,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        let require_size = self.offset + self.overlap_size;

        if self.buffer.len() == require_size {
            let buf = self.iter.next()?;
            buffer_overlapping_write(self.overlap_size, &mut self.buffer, buf.into_iter());
        }

        if self.buffer.len() <= require_size {
            panic!()
        }

        let item = self.buffer[self.offset].clone();

        self.offset += 1;
        if self.overlap_size < self.offset {
            self.buffer.drain(0..self.offset);
            self.offset = 0;
        }

        Some(item)
    }
}

pub fn buffer_overlapping_write<T: std::ops::Add<T, Output = T> + Clone>(
    overlap_size: usize,
    buffer: &mut Vec<T>,
    mut other: impl Iterator<Item = T>,
) {
    let len = buffer.len();
    for i in len - overlap_size..len {
        let Some(x) = other.next() else {
            return;
        };
        buffer[i] = buffer[i].clone() + x;
    }
    buffer.extend(other);
}
