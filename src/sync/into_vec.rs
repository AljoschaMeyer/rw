extern crate maybe_std as base;

use crate::maybe_uninit_slice_mut;
use base::num::NonZeroUsize;
use base::mem::MaybeUninit;
use base::vec::Vec;

use slice_n::Slice1;
use wrapper::Wrapper;

use crate::sync::*;

/// Collects data and can at any point be converted into a `Vec<T>.
pub struct IntoVec<T>(Vec<T>);

impl<T> IntoVec<T> {
    /// Create a new `IntoVec`.
    pub fn new() -> Self {
        IntoVec(Vec::new())
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> Consumer for IntoVec<T> {
    type Item = T;
    type Error = !;

    fn consume(&mut self, item: T) -> Result<(), Self::Error> {
        Ok(self.0.push(item))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T: Copy> BulkConsumer for IntoVec<T> {
    fn consumer_slots(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error> {
        if self.0.capacity() == self.0.len() {
            self.0.reserve(self.0.capacity());
        }

        Ok(unsafe { Slice1::from_slice_unchecked_mut(maybe_uninit_slice_mut(&mut self.0[..])) })
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.0.set_len(self.0.len() + amount.get());
    }
}

impl<T> Wrapper<Vec<T>> for IntoVec<T> {
    fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> AsRef<Vec<T>> for IntoVec<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> AsMut<Vec<T>> for IntoVec<T> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}
