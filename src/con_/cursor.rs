use core::convert::{AsRef, AsMut};
use core::num::NonZeroUsize;
use core::mem::MaybeUninit;

use slice_n::Slice1;
use wrapper::Wrapper;

use crate::con::{Consumer, BulkConsumer};
use crate::maybe_uninit_slice_mut;

/// Creates a consumes which places consumed data in the given slice.
pub fn cursor<'a, T>(s: &'a mut [T]) -> Cursor<'a, T> {
    Cursor(s, 0)
}

/// Consumes data into a mutable slice.
pub struct Cursor<'a, T>(&'a mut [T], usize);

impl<'a, T> Wrapper<&'a mut [T]> for Cursor<'a, T> {
    fn into_inner(self) -> &'a mut [T] {
        self.0
    }
}

impl<'a, T> AsRef<[T]> for Cursor<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<'a, T> AsMut<[T]> for Cursor<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0
    }
}

impl<'a, T> Consumer for Cursor<'a, T> {
    type Item = T;
    /// Emitted when the end of the slice has been reached.
    type Error = ();

    fn consume(&mut self, item: T) -> Result<(), Self::Error> {
        if self.0.len() == self.1 {
            Err(())
        } else {
            self.0[self.1] = item;
            self.1 += 1;
            Ok(())
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, T: Copy> BulkConsumer for Cursor<'a, T> {
    fn consumer_slots(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error> {
        if self.0.len() == self.1 {
            Err(())
        } else {
            Ok(unsafe { Slice1::from_slice_unchecked_mut(maybe_uninit_slice_mut(&mut self.0[self.1..])) })
        }
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.1 += amount.get();
    }
}
