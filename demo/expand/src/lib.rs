use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::Relaxed;
use std::sync::Mutex;

#[chaud::hot]
fn unit() {
    #[chaud::persist]
    static STATE: AtomicU32 = AtomicU32::new(42);

    STATE.fetch_add(1, Relaxed);
}

/// Hello, world.
#[chaud::hot]
#[cold]
pub(crate) fn single_with_attrs<'a>(x: &'a u32) -> u32 {
    *x + 1
}

#[chaud::hot]
pub fn multi<'a, 'b>(b: &'b bool, x: &'a u32, y: &'a u32) -> (&'a u32, &'b bool) {
    if *b { (x, b) } else { (y, b) }
}

pub struct Collector {
    buf: Mutex<Vec<String>>,
}

#[chaud::persist]
pub static ITEMS: Collector = {
    let buf = Mutex::new(vec![]);
    Collector { buf }
};

impl Collector {
    #[chaud::hot(self)]
    pub fn collect(this: &Collector, item: String) {
        this.buf.lock().unwrap().push(item);
    }
}
