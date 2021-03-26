extern crate maybe_std as base;

use base::alloc::{Allocator, Global};
use base::boxed::Box;
use base::num::NonZeroUsize;
use base::mem::MaybeUninit;

use slice_n::Slice1;

use crate::*;

/// A buffer holding up to a certain number of items.
#[derive(Debug)]
pub(crate) struct FixedBuffer<T, A = Global> where A: Allocator {
    data: Box<[MaybeUninit<T>], A>,
    // reading resumes from this position
    read: usize,
    // amount of valid data
    amount: usize,
}

impl<T> FixedBuffer<T> {
    pub fn new(capacity: NonZeroUsize) -> Self {
        FixedBuffer {
            data: Box::new_uninit_slice(capacity.get()),
            read: 0,
            amount: 0,
        }
    }
}

impl<T, A: Allocator> FixedBuffer<T, A> {
    pub fn new_in(capacity: NonZeroUsize, alloc: A) -> Self {
        FixedBuffer {
            data: Box::new_uninit_slice_in(capacity.get(), alloc),
            read: 0,
            amount: 0,
        }
    }

    pub fn get_amount(&self) -> usize {
        self.amount
    }

    pub fn get_capacity(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(self.data.len()) }
    }
}

impl<T: Copy, A: Allocator> FixedBuffer<T, A> {
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

    fn readable_fst(&mut self) -> &[MaybeUninit<T>] {
        if self.is_data_contiguous() {
            return &self.data[self.read..self.write_to()];
        } else {
            return &self.data[self.read..];
        }
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }

    fn write_to(&self) -> usize {
        (self.read + self.amount) % self.capacity()
    }
}

impl<T: Copy, A: Allocator> Consumer for FixedBuffer<T, A> {
    type Item = T;
    /// Emitted when there is currently no space for writing available.
    type Error = ();

    fn consume(&mut self, item: Self::Item) -> Result<(), Self::Error> {
        if self.amount == self.capacity() {
            return Err(());
        }

        self.data[self.write_to()].write(item);
        self.amount += 1;
        return Ok(());
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T: Copy, A: Allocator> BulkConsumer for FixedBuffer<T, A> {
    fn consumer_slots(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error> {
        if self.amount >= self.capacity() {
            return Err(());
        }

        Ok(unsafe { Slice1::from_slice_unchecked_mut(self.available_fst()) })
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.amount += amount.get();
    }
}

impl<T: Copy, A: Allocator> Producer for FixedBuffer<T, A> {
    type Item = T;
    /// Emitted when there are currently no items available.
    type Error = ();

    fn produce(&mut self) -> Result<Self::Item, Self::Error> {
        if self.amount == 0 {
            return Err(());
        }

        let old_r = self.read;
        self.read = (self.read + 1) % self.capacity();
        self.amount -= 1;
        return Ok(unsafe { self.data[old_r].assume_init() } );
    }

    fn slurp(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T: Copy, A: Allocator> BulkProducer for FixedBuffer<T, A> {
    fn producer_slots(&mut self) -> Result<& Slice1<Self::Item>, Self::Error> {
        if self.amount == 0 {
            return Err(());
        }

        Ok(unsafe { Slice1::from_slice_unchecked(MaybeUninit::slice_assume_init_ref(self.readable_fst())) })
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.read = (self.read + amount.get()) % self.capacity();
        self.amount -= amount.get();
    }
}
