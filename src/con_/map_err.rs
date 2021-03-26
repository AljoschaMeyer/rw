use core::convert::{AsRef, AsMut};
use core::num::NonZeroUsize;
use core::mem::MaybeUninit;

use slice_n::Slice1;
use wrapper::Wrapper;

use crate::{Consumer, BulkConsumer};

pub fn map_err<I, F>(inner: I, f: F) -> MapErr<I, F> {
    MapErr(inner, f)
}

pub struct MapErr<I, F>(I, F);

impl<I, F> Wrapper<I> for MapErr<I, F> {
    fn into_inner(self) -> I {
        self.0
    }
}

impl<I, F> AsRef<I> for MapErr<I, F> {
    fn as_ref(&self) -> &I {
        &self.0
    }
}

impl<I, F> AsMut<I> for MapErr<I, F> {
    fn as_mut(&mut self) -> &mut I {
        &mut self.0
    }
}

impl<I, F, T, E, E2> Consumer for MapErr<I, F> where
    I: Consumer<Item = T, Error = E>,
    F: Fn(E) -> E2
{
    type Item = T;
    type Error = E2;

    fn consume(&mut self, item: T) -> Result<(), Self::Error> {
        match self.0.consume(item) {
            Ok(_) => Ok(()),
            Err(e) => Err(self.1(e))
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        match self.0.flush() {
            Ok(_) => Ok(()),
            Err(e) => Err(self.1(e))
        }
    }
}

impl<I, F, T, E, E2> BulkConsumer for MapErr<I, F> where
    T: Copy,
    I: BulkConsumer<Item = T, Error = E>,
    F: Fn(E) -> E2
{
    fn consumer_slots(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error> {
        match self.0.consumer_slots() {
            Ok(s) => Ok(s),
            Err(e) => Err(self.1(e))
        }
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.0.did_consume(amount)
    }
}
