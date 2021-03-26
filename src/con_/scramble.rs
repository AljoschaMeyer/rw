use core::fmt::Debug;
use maybe_std::{
    boxed::Box,
    cmp::min,
    mem::MaybeUninit,
    num::NonZeroUsize,
};

use slice_n::Slice1;
use wrapper::Wrapper;

use arbitrary::{Arbitrary, Error, Unstructured};

use crate::*;
use ringbuffer::*;

#[derive(Debug, PartialEq, Eq, Arbitrary)]
pub enum ConsumeOperation {
    Consume,
    ConsumerSlots(NonZeroUsize),
    BulkConsume(NonZeroUsize),
    Flush,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConsumeOperations(Box<[ConsumeOperation]>);

impl ConsumeOperations {
    /// Checks that the operations contain at least one non-flush operation.
    pub fn new(operations: Box<[ConsumeOperation]>) -> Option<Self> {
        let mut found_non_flush = false;
        for op in operations.iter() {
            if *op != ConsumeOperation::Flush {
                found_non_flush = true;
                break;
            }
        }

        if found_non_flush {
            Some(ConsumeOperations(operations))
        } else {
            return None;
        }
    }
}

impl<'a> Arbitrary<'a> for ConsumeOperations {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        match Self::new(Arbitrary::arbitrary(u)?) {
            Some(ops) => Ok(ops),
            None => Err(Error::IncorrectFormat),
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <Box<[ConsumeOperation]> as Arbitrary<'a>>::size_hint(depth)
    }
}

#[derive(Debug)]
pub struct ScrambleConsumer<I, T> {
    inner: I,
    buf: FixedBuffer<T>,
    operations: Box<[ConsumeOperation]>,
    operations_index: usize,
}

impl<I, T> ScrambleConsumer<I, T> {
    pub fn new(inner: I, operations: ConsumeOperations, capacity: NonZeroUsize) -> Self {
        ScrambleConsumer {
            inner,
            buf: FixedBuffer::new(capacity),
            operations: operations.0,
            operations_index: 0,
        }
    }
}

impl<I: BulkConsumer<Item = T, Error = E>, T: Copy + Debug, E> Consumer for ScrambleConsumer<I, T> {
    type Item = T;
    type Error = E;

    fn consume(&mut self, item: Self::Item) -> Result<(), Self::Error> {
        while self.buf.get_capacity().get() == self.buf.get_amount() {
            self.perform_operation()?;
        }

        self.buf.consume(item).unwrap();
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        while self.buf.get_amount() > 0 {
            self.perform_operation()?;
        }
        self.inner.flush()
    }
}

impl<I: BulkConsumer<Item = T, Error = E>, T: Copy + Debug, E> BulkConsumer for ScrambleConsumer<I, T> {
    fn consumer_slots(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error> {
        while self.buf.get_capacity().get() == self.buf.get_amount() {
            self.perform_operation()?;
        }

        Ok(self.buf.consumer_slots().unwrap())
    }

    /// Tells the `BulkConsumer` that some amount of items has been placed in it. This must be
    /// accurate, because the `BulkConsumer` then assumes the corresponding memory to be
    /// initialized.
    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.buf.did_consume(amount)
    }
}

impl<I: BulkConsumer<Item = T, Error = E>, T: Copy + Debug, E> ScrambleConsumer<I, T> {
    fn perform_operation(&mut self) -> Result<(), E> {
        debug_assert!(self.buf.get_amount() > 0);

        match self.operations[self.operations_index] {
            ConsumeOperation::Consume => {
                self.inner.consume(self.buf.produce().unwrap())?;
            }
            ConsumeOperation::ConsumerSlots(n) => {
                let slots = self.inner.consumer_slots()?;
                let l = slots.len_();
                let slots = unsafe { Slice1::from_slice_unchecked_mut(&mut slots[..min(l, n.get())]) };
                let consume_amount = self.buf.bulk_produce(slots).unwrap();
                unsafe { self.inner.did_consume(consume_amount) };
            }
            ConsumeOperation::BulkConsume(n) => {
                let slots = self.buf.producer_slots().unwrap();
                let l = slots.len_();
                let slots = unsafe { Slice1::from_slice_unchecked(&slots[..min(l, n.get())]) };
                let consume_amount = self.inner.bulk_consume(slots)?;
                self.buf.did_produce(consume_amount);
            }
            ConsumeOperation::Flush => self.inner.flush()?,
        }

        self.operations_index = (self.operations_index + 1) % self.operations.len();
        Ok(())
    }
}

impl<I, T> Wrapper<I> for ScrambleConsumer<I, T> {
    fn into_inner(self) -> I {
        self.inner
    }
}

impl<I, T> AsRef<I> for ScrambleConsumer<I, T> {
    fn as_ref(&self) -> &I {
        &self.inner
    }
}

impl<I, T> AsMut<I> for ScrambleConsumer<I, T> {
    fn as_mut(&mut self) -> &mut I {
        &mut self.inner
    }
}
