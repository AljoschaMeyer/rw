extern crate maybe_std as base;

use base::num::NonZeroUsize;
use base::mem::MaybeUninit;
use base::cmp::min;
use base::vec::Vec;

use slice_n::Slice1;

use crate::sync::{Readable, Writable};

/// A buffer holding up to a certain number of items, and elastically allocating and deallocating
/// the memory for the buffer. Reading and writing `n` items from a buffer of size `m` is in
/// amortized `O(n)`, with a worst case of `O(m)`.
pub struct ElasticBuffer<T> {
    data: Vec<MaybeUninit<T>>,
    // reading resumes from this position
    read: usize,
    // amount of valid data
    amount: usize,
    max_size: usize,
    shrinking_threshold: usize,
}

impl<T> ElasticBuffer<T> {
    pub fn new(max_size: NonZeroUsize) -> Self {
        ElasticBuffer {
            data: Vec::new(),
            read: 0,
            amount: 0,
            max_size: max_size.get(),
            shrinking_threshold: 0,
        }
    }

    pub fn with_capacity(max_size: NonZeroUsize, capacity: usize) -> Self {
        ElasticBuffer {
            data: Vec::with_capacity(capacity),
            read: 0,
            amount: 0,
            max_size: max_size.get(),
            shrinking_threshold: 0,
        }
    }

    pub fn from_vec(max_size: NonZeroUsize, data: Vec<MaybeUninit<T>>) -> Self {
        ElasticBuffer {
            data,
            read: 0,
            amount: 0,
            max_size: max_size.get(),
            shrinking_threshold: 0,
        }
    }
}

impl<T: Copy> ElasticBuffer<T> {
    fn is_data_contiguous(&self) -> bool {
        self.read + self.amount < self.true_capacity()
    }

    fn available_fst(&mut self) -> &mut [MaybeUninit<T>] {
        let cap = self.true_capacity();
        if self.is_data_contiguous() {
            return &mut self.data[self.read + self.amount..cap];
        } else {
            return &mut self.data[(self.read + self.amount) % cap..self.read];
        };
    }

    fn available_snd(&mut self) -> &mut [MaybeUninit<T>] {
        if self.is_data_contiguous() {
            return &mut self.data[0..self.read];
        } else {
             return &mut self.data[0..0];
        };
    }

    fn needs_growing(&self) -> bool {
        self.amount == self.true_capacity()
    }

    fn grow(&mut self, size: usize) {
        debug_assert!(size <= self.max_size);
        debug_assert!(size > self.true_capacity());
        debug_assert!(size >= self.amount);
        let cap = self.true_capacity();

        if self.is_data_contiguous() {
            self.data.reserve_exact(size - cap);
        } else {
            let mut new_data = Vec::<MaybeUninit<T>>::with_capacity(size);
            new_data.extend_from_slice(self.available_fst());
            new_data.extend_from_slice(self.available_snd());
            self.data = new_data;
            self.read = 0;
        }
    }

    fn grow_if_needed(&mut self) {
        if self.needs_growing() {
            self.grow(min(self.true_capacity() * 2, self.max_size));
        }
    }

    fn needs_shrinking(&self) -> bool {
        self.amount * 2 < self.shrinking_threshold
    }

    fn shrink(&mut self, size: usize) {
        debug_assert!(size < self.true_capacity());
        debug_assert!(size >= self.amount);

        let mut new_data = Vec::<MaybeUninit<T>>::with_capacity(size);
        new_data.extend_from_slice(self.available_fst());
        new_data.extend_from_slice(self.available_snd());
        self.data = new_data;
        self.read = 0;
        self.shrinking_threshold = self.amount;
    }

    fn shrink_if_needed(&mut self) {
        if self.needs_shrinking() {
            self.shrink(self.amount);
        }
    }

    fn true_capacity(&self) -> usize {
        min(self.data.capacity(), self.max_size)
    }
}

impl<T: Copy> Writable for ElasticBuffer<T> {
    type Item = T;
    /// Emitted when there is currently no space for writing available.
    type Error = ();

    fn writable(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error> {
        if self.amount >= self.max_size {
            return Err(());
        }

        self.grow_if_needed();

        Ok(unsafe { Slice1::from_slice_unchecked_mut(self.available_fst()) })
    }

    unsafe fn wrote(&mut self, amount: NonZeroUsize) {
        self.amount += amount.get();
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T: Copy> Readable for ElasticBuffer<T> {
    type Item = T;
    /// Emitted when there are currently no items available.
    type Error = ();

    fn readable(&mut self) -> Result<& Slice1<Self::Item>, Self::Error> {
        if self.amount == 0 {
            return Err(());
        }

        Ok(unsafe { Slice1::from_slice_unchecked(MaybeUninit::slice_assume_init_ref(self.available_fst())) })
    }

    fn read(&mut self, amount: NonZeroUsize) {
        self.read = (self.read + amount.get()) % self.true_capacity();
        self.shrink_if_needed();
    }

    fn slurp(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
