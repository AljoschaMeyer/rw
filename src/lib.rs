#![cfg_attr(not(feature = "std"), feature(no_std))]
#![feature(maybe_uninit_write_slice)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_uninit_array)]
#![cfg_attr(all(any(feature = "alloc", feature = "std"), feature = "arbitrary"), feature(new_uninit))]
#![cfg_attr(any(feature = "alloc", feature = "std"), feature(allocator_api))]
#![feature(never_type)]

#[cfg(any(feature = "alloc", feature = "std"))]
mod ringbuffer;

pub mod pro;
use pro::{Producer, BulkProducer};

// Did you know that `con` is a reserved filename on Windows and everything breaks if you use it?
mod con_;
pub mod con {
    pub use super::con_::*;
}
use con::{Consumer, BulkConsumer};

use core::mem::MaybeUninit;

pub(crate) fn maybe_uninit_slice<'a, T>(s: &'a [T]) -> &'a [MaybeUninit<T>] {
    let ptr = s.as_ptr().cast::<MaybeUninit<T>>();
    unsafe { core::slice::from_raw_parts(ptr, s.len()) }
}

pub(crate) fn maybe_uninit_slice_mut<'a, T>(s: &'a mut [T]) -> &'a mut [MaybeUninit<T>] {
    let ptr = s.as_mut_ptr().cast::<MaybeUninit<T>>();
    unsafe { core::slice::from_raw_parts_mut(ptr, s.len()) }
}

/// Pipes all items from the `Producer` into the `Consumer`. Does neither flush nor slurp.
pub fn pipe<P, C, T, E>(p: &mut P, c: &mut C) -> E where
    P: Producer<Item = T, Error = E>,
    C: Consumer<Item = T, Error = E>,
{
    loop {
        match p.produce() {
            Ok(item) => match c.consume(item) {
                Ok(()) => {}
                Err(e) => return e,
            }
            Err(e) => return e,
        }
    }
}

/// Writes all items from the `BulkProducer` to the `BulkConsumer`. Does neither flush nor slurp.
pub fn bulk_produce_all<P, C, T, E>(p: &mut P, c: &mut C) -> E where
    T: Copy,
    P: BulkProducer<Item = T, Error = E>,
    C: BulkConsumer<Item = T, Error = E>,
{
    loop {
        match c.consumer_slots() {
            Ok(s) => match p.bulk_produce(s) {
                Ok(amount) => unsafe { c.did_consume(amount) },
                Err(e) => return e,
            }
            Err(e) => return e,
        }
    }
}

/// Reads all items from the `BulkProducer` into the `BulkConsumer`. Does neither flush nor slurp.
pub fn bulk_consume_all<P, C, T, E>(p: &mut P, c: &mut C) -> E where
    T: Copy,
    P: BulkProducer<Item = T, Error = E>,
    C: BulkConsumer<Item = T, Error = E>,
{
    loop {
        match p.producer_slots() {
            Ok(s) => match c.bulk_consume(s) {
                Ok(amount) => p.did_produce(amount),
                Err(e) => return e,
            }
            Err(e) => return e,
        }
    }
}
