#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

use core::cmp::min;
use core::num::NonZeroUsize;

use wrapper::Wrapper;

use rw::sync::*;

fuzz_target!(|data: &[u8]| {
    match <(Box<[u8]>, ConsumeOperations, NonZeroUsize)>::arbitrary(&mut Unstructured::new(data)) {
        Ok((a, ops, cap)) => {
            let cap = NonZeroUsize::new(min(cap.get(), 2048)).unwrap();
            let mut o = CursorOut::new(&a[..]);
            let mut i = ScrambleConsumer::new(IntoVec::new(), ops, cap);

            assert_eq!(bulk_consume_all(&mut o, &mut i), ());
            let _ = i.flush();

            assert_eq!(&i.as_ref().as_ref()[..], &o.as_ref()[..]);
        }
        _ => {}
    }
});
