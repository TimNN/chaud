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
            unsafe extern "Rust" {
                #[link_name = "_CHAUD::expand::STATE"]
                safe static STATE: AtomicU32;
            }
            const _: bool = ::core::option::Option::Some("1").is_some();

            STATE.fetch_add(1, Relaxed);
        };
    }
    unsafe extern "Rust" {
        #[link_name = "_CHAUD::expand::unit"]
        safe static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func>;
    }
    const _: bool = ::core::option::Option::Some("1").is_some();

    #[allow(unused)]
    fn __chaud__reload() {
        #[allow(unsafe_code)]
        {
            #[used]
            #[allow(non_upper_case_globals, non_snake_case)]
            #[doc(hidden)]
            static f: extern "C" fn() -> usize = {
                #[allow(non_snake_case)]
                extern "C" fn f() -> usize {
                    unsafe {
                        __chaud__reload();
                        0
                    }
                }
                f
            };
        }
        {
            __chaud_FUNC.update();
        }
    }
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
    unsafe extern "Rust" {
        #[link_name = "_CHAUD::expand::single_with_attrs"]
        safe static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func>;
    }
    const _: bool = ::core::option::Option::Some("1").is_some();
    #[allow(unused)]
    fn __chaud__reload() {
        #[allow(unsafe_code)]
        {
            #[used]
            #[allow(non_upper_case_globals, non_snake_case)]
            #[doc(hidden)]
            static f: extern "C" fn() -> usize = {
                #[allow(non_snake_case)]
                extern "C" fn f() -> usize {
                    unsafe {
                        __chaud__reload();
                        0
                    }
                }
                f
            };
        }
        {
            __chaud_FUNC.update();
        }
    }
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
    unsafe extern "Rust" {
        #[link_name = "_CHAUD::expand::multi"]
        safe static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func>;
    }
    const _: bool = ::core::option::Option::Some("1").is_some();
    #[allow(unused)]
    fn __chaud__reload() {
        #[allow(unsafe_code)]
        {
            #[used]
            #[allow(non_upper_case_globals, non_snake_case)]
            #[doc(hidden)]
            static f: extern "C" fn() -> usize = {
                #[allow(non_snake_case)]
                extern "C" fn f() -> usize {
                    unsafe {
                        __chaud__reload();
                        0
                    }
                }
                f
            };
        }
        {
            __chaud_FUNC.update();
        }
    }
    __chaud_FUNC.get()(p0, p1, p2)
}
pub struct Collector {
    buf: Mutex<Vec<String>>,
}
unsafe extern "Rust" {
    #[link_name = "_CHAUD::expand::ITEMS"]
    safe static ITEMS: Collector;
}
const _: bool = ::core::option::Option::Some("1").is_some();
impl Collector {
    #[inline]
    pub fn collect(self: &Collector, p1: String) {
        struct __chaud_func;
        unsafe impl ::chaud::__internal::Func for __chaud_func {
            type Ptr = fn(&Collector, String);
            const NAME: &'static str = "expand::collect";
            const actual: Self::Ptr = |this, item| {
                this.buf.lock().unwrap().push(item);
            };
        }
        unsafe extern "Rust" {
            #[link_name = "_CHAUD::expand::collect"]
            safe static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func>;
        }
        const _: bool = ::core::option::Option::Some("1").is_some();
        #[allow(unused)]
        fn __chaud__reload() {
            #[allow(unsafe_code)]
            {
                #[used]
                #[allow(non_upper_case_globals, non_snake_case)]
                #[doc(hidden)]
                static f: extern "C" fn() -> usize = {
                    #[allow(non_snake_case)]
                    extern "C" fn f() -> usize {
                        unsafe {
                            __chaud__reload();
                            0
                        }
                    }
                    f
                };
            }
            {
                __chaud_FUNC.update();
            }
        }
        __chaud_FUNC.get()(self, p1)
    }
}
