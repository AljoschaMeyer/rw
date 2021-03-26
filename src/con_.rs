use core::num::NonZeroUsize;
use core::mem::MaybeUninit;
use core::cmp::min;

use slice_n::Slice1;

mod cursor;
pub use cursor::*;

mod map_err;
pub use map_err::*;

#[cfg(all(feature = "alloc", feature = "arbitrary"))]
mod scramble;
#[cfg(all(feature = "alloc", feature = "arbitrary"))]
pub use scramble::*;

/// A `Consumer` consumes items one by one.
pub trait Consumer {
    /// The type of values that are consumed by the `Consumer`.
    type Item;
    /// Everything that can go wrong. After any method has returned an error, all further method
    /// calls have unspecified semantics.
    type Error;

    /// Consumes a single item.
    fn consume(&mut self, item: Self::Item) -> Result<(), Self::Error>;

    /// A `Consumer` is allowed to store written data in a buffer without immediately processing
    /// it. This method triggers immediate processing of all currently buffered data.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// A `BulkConsumer` can consume multiple pieces of copyable data at a time.
pub trait BulkConsumer: Consumer where Self::Item: Copy {
    /// Returns a nonempty buffer into which items can be placed. The memory in the buffer is not
    /// necessarily initialized.
    fn consumer_slots(&mut self) -> Result<&mut Slice1<MaybeUninit<Self::Item>>, Self::Error>;

    /// Tells the `BulkConsumer` that some amount of items has been placed in it. This must be
    /// accurate, because the `BulkConsumer` then assumes the corresponding memory to be
    /// initialized.
    unsafe fn did_consume(&mut self, amount: NonZeroUsize);

    /// The `BulkConsumer` consumes a non-zero number of items from the provided buffer, and
    /// returns how many it has consumed.
    fn bulk_consume(&mut self, data: &Slice1<Self::Item>) -> Result<NonZeroUsize, Self::Error> {
        let l = self.consumer_slots()?;
        let amount = min(l.len_(), data.len_());
        MaybeUninit::write_slice(&mut l[..amount], &data[..amount]);
        unsafe {
            let amount = NonZeroUsize::new_unchecked(amount);
            self.did_consume(amount);
            Ok(amount)
        }
    }
}
