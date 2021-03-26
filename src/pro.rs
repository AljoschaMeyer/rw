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

/// A `Producer` produces items one by one.
pub trait Producer {
    /// The type of values that are produced by the `Producer`.
    type Item;
    /// Everything that can go wrong. After any method has returned an error, all further method
    /// calls have unspecified semantics.
    type Error;

    /// Produces a single item.
    fn produce(&mut self) -> Result<Self::Item, Self::Error>;

    /// A `Producer` is allowed to obtain data from some data source and buffer it even before it
    /// is requested to be produced. This method instructs the `Producer` to move as much data from
    /// the data source into the internal buffer as possible.
    fn slurp(&mut self) -> Result<(), Self::Error>;
}

/// A `BulkProducer` can produce multiple pieces of copyable data at a time.
pub trait BulkProducer: Producer where Self::Item: Copy {
    /// Returns a nonempty buffer from which items can be taken.
    fn producer_slots(&mut self) -> Result<&Slice1<Self::Item>, Self::Error>;

    /// Tells the `BulkProducer` that some amount of items has been taken from it.
    fn did_produce(&mut self, amount: NonZeroUsize);

    /// The `BulkProducer` produces a non-zero number of items into the provided buffer, and
    /// returns how many it has produced. The memory in the buffer does not need to be initialized.
    fn bulk_produce(&mut self, buffer: &mut Slice1<MaybeUninit<Self::Item>>) -> Result<NonZeroUsize, Self::Error> {
        let r = self.producer_slots()?;
        let amount = min(r.len_(), buffer.len_());
        MaybeUninit::write_slice(&mut buffer[..amount], &r[..amount]);
        unsafe {
            let amount = NonZeroUsize::new_unchecked(amount);
            self.did_produce(amount);
            Ok(amount)
        }
    }
}
