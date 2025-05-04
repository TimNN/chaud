#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::Relaxed;
use std::sync::Mutex;

#[inline]
fn unit() {
    struct __chaud_func;
    unsafe impl ::chaud::__internal::Func for __chaud_func {
        type Ptr = fn();
        const NAME: &'static str = "expand::unit";
        const actual: Self::Ptr = || {
            #[unsafe(export_name = "_CHAUD::expand::STATE")]
            static STATE: AtomicU32 = AtomicU32::new(42);

            STATE.fetch_add(1, Relaxed);
        };
    }
    #[unsafe(export_name = "_CHAUD::expand::unit")]
    static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func> =
        ::chaud::__internal::FuncStorage::new();
    __chaud_FUNC.get()()
}

#[doc = " Hello, world."]
#[cold]
#[inline]
pub(crate) fn single_with_attrs<'a>(p0: &'a u32) -> u32 {
    struct __chaud_func;
    unsafe impl ::chaud::__internal::Func for __chaud_func {
        type Ptr = for<'a> fn(&'a u32) -> u32;
        const NAME: &'static str = "expand::single_with_attrs";
        const actual: Self::Ptr = |x| *x + 1;
    }
    #[unsafe(export_name = "_CHAUD::expand::single_with_attrs")]
    static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func> =
        ::chaud::__internal::FuncStorage::new();
    __chaud_FUNC.get()(p0)
}

#[inline]
pub fn multi<'a, 'b>(p0: &'b bool, p1: &'a u32, p2: &'a u32) -> (&'a u32, &'b bool) {
    struct __chaud_func;
    unsafe impl ::chaud::__internal::Func for __chaud_func {
        type Ptr = for<'a, 'b> fn(&'b bool, &'a u32, &'a u32) -> (&'a u32, &'b bool);
        const NAME: &'static str = "expand::multi";
        const actual: Self::Ptr = |b, x, y| {
            if *b { (x, b) } else { (y, b) }
        };
    }
    #[unsafe(export_name = "_CHAUD::expand::multi")]
    static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func> =
        ::chaud::__internal::FuncStorage::new();
    __chaud_FUNC.get()(p0, p1, p2)
}

pub struct Collector {
    buf: Mutex<Vec<String>>,
}

#[unsafe(export_name = "_CHAUD::expand::ITEMS")]
static ITEMS: Collector = {
    let buf = Mutex::new(::alloc::vec::Vec::new());
    Collector { buf }
};
impl Collector {
    #[inline]
    pub fn collect(self: &Self, p1: String) {
        struct __chaud_func;
        unsafe impl ::chaud::__internal::Func for __chaud_func {
            type Ptr = fn(&Self, String);
            const NAME: &'static str = "expand::collect";
            const actual: Self::Ptr = |this, item| {
                this.buf.lock().unwrap().push(item);
            };
        }
        #[unsafe(export_name = "_CHAUD::expand::collect")]
        static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func> =
            ::chaud::__internal::FuncStorage::new();
        __chaud_FUNC.get()(self, p1)
    }
}
