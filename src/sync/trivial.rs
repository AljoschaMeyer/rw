use core::num::NonZeroUsize;
use core::mem::MaybeUninit;

use slice_n::Slice1;

use crate::sync::{Readable, Writable};

/// A `Writable` that swallows data without doing anything with it.
pub struct TrivialWritable<T>([MaybeUninit<T>; 1]);

impl<T> TrivialWritable<T> {
    /// Create a new `TrivialWritable`.
    pub fn new() -> Self {
        TrivialWritable([MaybeUninit::uninit()])
    }
}

impl<T: Copy> Writable for TrivialWritable<T> {
    type Item = T;
    type Error = !;

    fn writable(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error> {
        unsafe { Ok(Slice1::from_slice_unchecked_mut(&mut self.0[..])) }
    }

    unsafe fn wrote(&mut self, _amount: NonZeroUsize) {}

    fn read(&mut self, data: &Slice1<Self::Item>) -> Result<NonZeroUsize, Self::Error> {
        Ok(data.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A `Readable` that repeats the same item forever.
pub struct TrivialReadable<T>([T; 1]);

impl<T> TrivialReadable<T> {
    /// Create a new `TrivialReadable`.
    pub fn new(item: T) -> Self {
        TrivialReadable([item])
    }
}

impl<T: Copy> Readable for TrivialReadable<T> {
    type Item = T;
    type Error = !;

    fn readable(&mut self) -> Result<& Slice1<Self::Item>, Self::Error> {
        unsafe { Ok(Slice1::from_slice_unchecked(&mut self.0[..])) }
    }

    fn read(&mut self, _amount: NonZeroUsize) {}

    fn write(&mut self, buffer: &mut Slice1<MaybeUninit<Self::Item>>) -> Result<NonZeroUsize, Self::Error> {
        let len = buffer.len_();
        for i in 0..len {
            buffer[i].write(self.0[0]);
        }
        unsafe { Ok(NonZeroUsize::new_unchecked(len)) }
    }

    fn slurp(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
