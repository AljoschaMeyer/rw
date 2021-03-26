extern crate maybe_std as base;

use base::num::NonZeroUsize;
use base::boxed::Box;

use slice_n::Slice1;

use crate::sync::*;

/// A readable that repeatedly outputs the same data.
pub struct Repeat<T>(Box<Slice1<T>>, usize);

impl<'a, T> Repeat<T> {
    /// Create a new `Repeat`, endlessly repeating the given data.
    pub fn new(data: Box<Slice1<T>>) -> Self {
        Repeat(data, 0)
    }
}

impl<T: Clone> Producer for Repeat<T> {
    type Item = T;
    type Error = !;

    fn produce(&mut self) -> Result<T, !> {
        let old_index = self.1;
        self.1 = (self.1 + 1) % self.0.len_();
        Ok(self.0[old_index].clone())
    }

    fn slurp(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T: Copy> BulkProducer for Repeat<T> {
    fn producer_slots(&mut self) -> Result<& Slice1<Self::Item>, Self::Error> {
        Ok(unsafe { Slice1::from_slice_unchecked(&self.0[self.1..]) })
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.1 = (self.1 + amount.get()) % self.0.len_();
    }
}
