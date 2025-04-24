use crate::hot::handle::{ErasedFnPtr, ErasedHandle};
use jiff::Timestamp;
use std::ffi::CString;

enum State {
    Active,
    Mangled(CString),
    Loaded(ErasedFnPtr),
}

pub struct TrackedSymbol {
    handle: ErasedHandle,
    mtime: Timestamp,
    state: State,
}
