#![no_main]
use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};

use core::cmp::min;

use rw::sync::*;

fuzz_target!(|data: &[u8]| {
    match <(Box<[u8]>, Box<[u8]>)>::arbitrary(&mut Unstructured::new(data)) {
        Ok((a, mut b)) => {
            let mut o = CursorOut::new(&a[..]);
            let mut i = CursorIn::new(&mut b[..]);

            assert_eq!(pipe(&mut o, &mut i), ());

            let m = min(o.as_ref().len(), i.as_ref().len());
            assert_eq!(&i.as_ref()[..m], &o.as_ref()[..m]);
        }
        _ => {}
    }
});
