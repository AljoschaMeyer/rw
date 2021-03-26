use core::fmt::Debug;
use maybe_std::{
    boxed::Box,
    cmp::min,
    num::NonZeroUsize,
};

use slice_n::Slice1;
use wrapper::Wrapper;

use arbitrary::{Arbitrary, Error, Unstructured};

use crate::*;
use ringbuffer::*;

#[derive(Debug, PartialEq, Eq, Arbitrary)]
pub enum ProduceOperation {
    Produce,
    ProducerSlots(NonZeroUsize),
    BulkProduce(NonZeroUsize),
    Slurp,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProduceOperations(Box<[ProduceOperation]>);

impl ProduceOperations {
    /// Checks that the operations contain at least one non-slurp operation.
    pub fn new(operations: Box<[ProduceOperation]>) -> Option<Self> {
        let mut found_non_slurp = false;
        for op in operations.iter() {
            if *op != ProduceOperation::Slurp {
                found_non_slurp = true;
                break;
            }
        }

        if found_non_slurp {
            Some(ProduceOperations(operations))
        } else {
            return None;
        }
    }
}

impl<'a> Arbitrary<'a> for ProduceOperations {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        match Self::new(Arbitrary::arbitrary(u)?) {
            Some(ops) => Ok(ops),
            None => Err(Error::IncorrectFormat),
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <Box<[ProduceOperation]> as Arbitrary<'a>>::size_hint(depth)
    }
}

#[derive(Debug)]
pub struct ScrambleProducer<I, T, E> {
    inner: I,
    buf: FixedBuffer<T>,
    err: Option<E>,
    operations: Box<[ProduceOperation]>,
    operations_index: usize,
}

impl<I, T, E> ScrambleProducer<I, T, E> {
    pub fn new(inner: I, operations: ProduceOperations, capacity: NonZeroUsize) -> Self {
        ScrambleProducer {
            inner,
            buf: FixedBuffer::new(capacity),
            err: None,
            operations: operations.0,
            operations_index: 0,
        }
    }
}

impl<I: BulkProducer<Item = T, Error = E>, T: Copy + Debug, E> Producer for ScrambleProducer<I, T, E> {
    type Item = T;
    type Error = E;

    fn produce(&mut self) -> Result<Self::Item, Self::Error> {
        if self.buf.get_amount() == 0 && self.err.is_some() {
            return Err(self.err.take().unwrap());
        }

        while self.buf.get_amount() == 0 {
            self.perform_operation()?;
        }

        Ok(self.buf.produce().unwrap())
    }

    fn slurp(&mut self) -> Result<(), Self::Error> {
        if self.buf.get_amount() == 0 && self.err.is_some() {
            return Err(self.err.take().unwrap());
        }

        while self.buf.get_amount() < self.buf.get_capacity().get() {
            match self.perform_operation() {
                Ok(_) => {}
                Err(e) => {
                    self.err = Some(e);
                    return Ok(());
                }
            }
        }
        self.inner.slurp()
    }
}

impl<I: BulkProducer<Item = T, Error = E>, T: Copy + Debug, E> BulkProducer for ScrambleProducer<I, T, E> {
    fn producer_slots(&mut self) -> Result<&Slice1<Self::Item>, Self::Error> {
        if self.buf.get_amount() == 0 && self.err.is_some() {
            return Err(self.err.take().unwrap());
        }

        while self.buf.get_amount() == 0 {
            self.perform_operation()?;
        }

        Ok(self.buf.producer_slots().unwrap())
    }

    /// Tells the `BulkConsumer` that some amount of items has been placed in it. This must be
    /// accurate, because the `BulkConsumer` then assumes the corresponding memory to be
    /// initialized.
    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.buf.did_produce(amount)
    }
}

impl<I: BulkProducer<Item = T, Error = E>, T: Copy + Debug, E> ScrambleProducer<I, T, E> {
    fn perform_operation(&mut self) -> Result<(), E> {
        debug_assert!(self.buf.get_amount() < self.buf.get_capacity().get());

        match self.operations[self.operations_index] {
            ProduceOperation::Produce => {
                self.buf.consume(self.inner.produce()?).unwrap();
            }
            ProduceOperation::ProducerSlots(n) => {
                let slots = self.inner.producer_slots()?;
                let l = slots.len_();
                let slots = unsafe { Slice1::from_slice_unchecked(&slots[..min(l, n.get())]) };
                let consume_amount = self.buf.bulk_consume(slots).unwrap();
                self.inner.did_produce(consume_amount);
            }
            ProduceOperation::BulkProduce(n) => {
                let slots = self.buf.consumer_slots().unwrap();
                let l = slots.len_();
                let slots = unsafe { Slice1::from_slice_unchecked_mut(&mut slots[..min(l, n.get())]) };
                let consume_amount = self.inner.bulk_produce(slots)?;
                unsafe { self.buf.did_consume(consume_amount) };
            }
            ProduceOperation::Slurp => self.inner.slurp()?,
        }

        self.operations_index = (self.operations_index + 1) % self.operations.len();
        Ok(())
    }
}

impl<I, T, E> Wrapper<I> for ScrambleProducer<I, T, E> {
    fn into_inner(self) -> I {
        self.inner
    }
}

impl<I, T, E> AsRef<I> for ScrambleProducer<I, T, E> {
    fn as_ref(&self) -> &I {
        &self.inner
    }
}

impl<I, T, E> AsMut<I> for ScrambleProducer<I, T, E> {
    fn as_mut(&mut self) -> &mut I {
        &mut self.inner
    }
}
