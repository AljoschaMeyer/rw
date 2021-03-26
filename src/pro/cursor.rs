use core::convert::AsRef;
use core::num::NonZeroUsize;

use slice_n::Slice1;
use wrapper::Wrapper;

use crate::pro::*;

/// Creates a producer which produces the data in the given slice.
pub fn cursor<'a, T>(s: &'a [T]) -> Cursor<'a, T> {
    Cursor(s, 0)
}

/// Produces data from a slice.
pub struct Cursor<'a, T>(&'a [T], usize);

impl<'a, T> Wrapper<&'a [T]> for Cursor<'a, T> {
    fn into_inner(self) -> &'a [T] {
        self.0
    }
}

impl<'a, T> AsRef<[T]> for Cursor<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<'a, T: Clone> Producer for Cursor<'a, T> {
    type Item = T;
    /// Emitted when the end of the slice has been reached.
    type Error = ();

    fn produce(&mut self) -> Result<T, Self::Error> {
        if self.0.len() == self.1 {
            Err(())
        } else {
            let item = self.0[self.1].clone();
            self.1 += 1;
            Ok(item)
        }
    }

    fn slurp(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, T: Copy> BulkProducer for Cursor<'a, T> {
    fn producer_slots(&mut self) -> Result<& Slice1<Self::Item>, Self::Error> {
        Slice1::from_slice(&self.0[self.1..]).ok_or(())
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.1 += amount.get();
    }
}
