#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

use core::cmp::min;
use core::num::NonZeroUsize;

use wrapper::Wrapper;

use rw::sync::*;

fuzz_target!(|data: &[u8]| {
    match <(Box<[u8]>, Box<[u8]>, ConsumeOperations, ConsumeOperations, NonZeroUsize, NonZeroUsize)>::arbitrary(&mut Unstructured::new(data)) {
        Ok((a, mut b, ops_a, ops_b, cap_a, cap_b)) => {
            if b.len() < a.len() {
                return;
            }
            let cap_a = NonZeroUsize::new(min(cap_a.get(), 2048)).unwrap();
            let cap_b = NonZeroUsize::new(min(cap_b.get(), 2048)).unwrap();
            let mut o = CursorOut::new(&a[..]);
            let mut i = ScrambleConsumer::new(
                ScrambleConsumer::new(
                    CursorIn::new(&mut b[..]),
                    ops_b, cap_b),
                ops_a, cap_a
            );


            assert_eq!(bulk_consume_all(&mut o, &mut i), ());
            let _ = i.flush();

            let i = i.into_inner().into_inner();
            let m = min(o.as_ref().len(), i.as_ref().len());
            assert_eq!(&i.as_ref()[..m], &o.as_ref()[..m]);
        }
        _ => {}
    }
});
