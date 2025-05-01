#![doc = include_str!("../README.md")]
#![allow(dead_code, reason = "TODO: remove once module is fully in use")]

pub use ctor::declarative::ctor;

pub use self::func::{Func, FuncStorage};

mod func;
