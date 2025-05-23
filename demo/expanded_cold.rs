#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::Relaxed;
use std::sync::Mutex;

fn unit() {
    static STATE: AtomicU32 = AtomicU32::new(42);

    STATE.fetch_add(1, Relaxed);
}

#[doc = " Hello, world."]
#[cold]
pub(crate) fn single_with_attrs<'a>(x: &'a u32) -> u32 {
    *x + 1
}

pub fn multi<'a, 'b>(b: &'b bool, x: &'a u32, y: &'a u32) -> (&'a u32, &'b bool) {
    if *b { (x, b) } else { (y, b) }
}

pub struct Collector {
    buf: Mutex<Vec<String>>,
}

static ITEMS: Collector = {
    let buf = Mutex::new(::alloc::vec::Vec::new());
    Collector { buf }
};
impl Collector {
    pub fn collect(self: &Collector, item: String) {
        let this = self;
        this.buf.lock().unwrap().push(item);
    }
}
