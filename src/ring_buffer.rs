pub struct RingBuffer<T: Clone + Copy> {
    pub buffer: Vec<T>,
    pub capacity: usize,
    pub read_pos: usize,
    pub write_pos: usize,
}

impl<T: Clone + Copy> RingBuffer<T> {
    pub fn new(capacity: usize, value: T) -> RingBuffer<T> {
        RingBuffer {
            buffer: vec![value; capacity + 1],
            capacity,
            read_pos: 0,
            write_pos: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn num_readable(&self) -> usize {
        if self.read_pos <= self.write_pos {
            self.write_pos - self.read_pos
        } else {
            self.write_pos + (self.capacity + 1) - self.read_pos
        }
    }

    pub fn num_writable(&self) -> usize {
        self.capacity - self.num_readable()
    }

    pub fn is_full(&self) -> bool {
        self.num_writable() == 0
    }

    pub fn is_empty(&self) -> bool {
        self.num_readable() == 0
    }

    pub fn read(&mut self, dest: &mut [T]) -> bool {
        let length = dest.len();

        if self.num_readable() < length {
            return false;
        }

        let buffer_length = self.capacity + 1;
        let copy1 = std::cmp::min(buffer_length - self.read_pos, length);

        dest[0..copy1].clone_from_slice(&self.buffer[self.read_pos..self.read_pos + copy1]);

        if copy1 == length {
            return true;
        }

        let copy2 = std::cmp::max(length, copy1) - copy1;
        dest[copy1..copy1 + copy2].clone_from_slice(&self.buffer[0..copy2]);

        true
    }

    pub fn write(&mut self, src: &[T]) -> bool {
        let length = src.len();

        if self.num_writable() < length {
            return false;
        }

        let buffer_length = self.capacity + 1;
        let copy1 = std::cmp::min(buffer_length - self.write_pos, length);

        self.buffer[self.write_pos..self.write_pos + copy1].clone_from_slice(&src[0..copy1]);

        if copy1 == length {
            self.write_pos += copy1;
            return true;
        }

        let copy2 = length - copy1;
        self.buffer[0..copy2].clone_from_slice(&src[copy1..copy1 + copy2]);
        self.write_pos = copy2;
        true
    }

    pub fn fill(&mut self, length: usize, value: T) -> bool {
        if self.num_writable() < length {
            return false;
        }

        let buffer_length = self.capacity + 1;
        let copy1 = std::cmp::min(buffer_length - self.write_pos, length);

        self.buffer[self.write_pos..self.write_pos + copy1].fill(value.clone());

        if copy1 == length {
            self.write_pos += copy1;
            return true;
        }

        let copy2 = buffer_length - copy1;
        self.buffer[0..copy2].fill(value);
        self.write_pos = copy2;

        true
    }

    pub fn discard(&mut self, length: usize) -> bool {
        if self.num_readable() < length {
            return false;
        }

        let buffer_length = self.capacity + 1;
        let discard1 = std::cmp::min(buffer_length - self.read_pos, length);

        if discard1 == length {
            self.read_pos += length;
            return true;
        }

        let discard2 = length - discard1;
        self.read_pos = discard2;

        true
    }

    pub fn discard_all(&mut self) {
        self.discard(self.num_readable());
    }
}

pub trait OverlappedAddable<T> {
    fn overlap_add(&mut self, src: &[T], overlap_size: usize) -> bool;
}

impl<T> OverlappedAddable<T> for RingBuffer<T>
where
    T: std::ops::AddAssign + Clone + Copy,
{
    fn overlap_add(&mut self, src: &[T], overlap_size: usize) -> bool {
        let length = src.len();

        if length < overlap_size {
            return false;
        }

        if self.num_readable() < overlap_size {
            return false;
        }

        let num_to_write_new = length - overlap_size;
        if self.num_writable() < num_to_write_new {
            return false;
        }

        let buffer_length = self.capacity + 1;

        let write_start = if self.write_pos >= overlap_size {
            self.write_pos - overlap_size
        } else {
            buffer_length - (overlap_size - self.write_pos)
        };

        let copy1 = std::cmp::min(buffer_length - write_start, overlap_size);

        for i in 0..copy1 {
            self.buffer[write_start + i] += src[i];
        }

        if copy1 != overlap_size {
            let copy2 = overlap_size - copy1;
            for i in 0..copy2 {
                self.buffer[i] += src[copy1 + i];
            }
        }

        self.write(&src[overlap_size..length])
    }
}
