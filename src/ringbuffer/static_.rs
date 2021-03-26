extern crate maybe_std as base;

use base::num::NonZeroUsize;
use base::mem::MaybeUninit;

use slice_n::Slice1;

use crate::sync::{Readable, Writable};

/// A buffer holding up to a certain, statically determined number of items.
pub struct StaticBuffer<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    // reading resumes from this position
    read: usize,
    // amount of valid data
    amount: usize,
}

impl<T, const N: usize> StaticBuffer<T, N> {
    pub fn new() -> Self {
        StaticBuffer {
            data: MaybeUninit::uninit_array(),
            read: 0,
            amount: 0,
        }
    }

    pub fn from_array(data: [MaybeUninit<T>; N]) -> Self {
        StaticBuffer {
            data,
            read: 0,
            amount: 0,
        }
    }
}

impl<T: Copy, const N: usize> StaticBuffer<T, N> {
    fn is_data_contiguous(&self) -> bool {
        self.read + self.amount < self.capacity()
    }

    fn available_fst(&mut self) -> &mut [MaybeUninit<T>] {
        let cap = self.capacity();
        if self.is_data_contiguous() {
            return &mut self.data[self.read + self.amount..cap];
        } else {
            return &mut self.data[(self.read + self.amount) % cap..self.read];
        };
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }
}

impl<T: Copy, const N: usize> Writable for StaticBuffer<T, N> {
    type Item = T;
    /// Emitted when there is currently no space for writing available.
    type Error = ();

    fn writable(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error> {
        if self.amount >= self.capacity() {
            return Err(());
        }

        Ok(unsafe { Slice1::from_slice_unchecked_mut(self.available_fst()) })
    }

    unsafe fn wrote(&mut self, amount: NonZeroUsize) {
        self.amount += amount.get();
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T: Copy, const N: usize> Readable for StaticBuffer<T, N> {
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
        self.read = (self.read + amount.get()) % self.capacity();
    }

    fn slurp(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
