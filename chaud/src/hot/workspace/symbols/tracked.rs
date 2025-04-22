use crate::hot::handle::{ErasedFnPtr, ErasedHandle};
use std::ffi::CString;
use std::time::SystemTime;

enum State {
    Active,
    Mangled(CString),
    Loaded(ErasedFnPtr),
}

pub struct TrackedSymbol {
    handle: ErasedHandle,
    mtime: SystemTime,
    state: State,
}
