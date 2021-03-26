use core::convert::{AsRef, AsMut};
use core::num::NonZeroUsize;

use slice_n::Slice1;
use wrapper::Wrapper;

use crate::{Producer, BulkProducer};

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

impl<I, F, T, E, E2> Producer for MapErr<I, F> where
    I: Producer<Item = T, Error = E>,
    F: Fn(E) -> E2
{
    type Item = T;
    type Error = E2;

    fn produce(&mut self) -> Result<T, Self::Error> {
        match self.0.produce() {
            Ok(item) => Ok(item),
            Err(e) => Err(self.1(e))
        }
    }

    fn slurp(&mut self) -> Result<(), Self::Error> {
        match self.0.slurp() {
            Ok(_) => Ok(()),
            Err(e) => Err(self.1(e))
        }
    }
}

impl<I, F, T, E, E2> BulkProducer for MapErr<I, F> where
    T: Copy,
    I: BulkProducer<Item = T, Error = E>,
    F: Fn(E) -> E2
{
    fn producer_slots(&mut self) -> Result<&Slice1<Self::Item>, Self::Error> {
        match self.0.producer_slots() {
            Ok(s) => Ok(s),
            Err(e) => Err(self.1(e))
        }
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.0.did_produce(amount)
    }
}
