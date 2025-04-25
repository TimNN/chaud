//! Representation of a [`Handle`][crate::Handle] when hot-reloading is enabled.
//!
//! # Definitions
//!
//! * An **"erased"** function pointer is a function pointer that has been
//!   transmuted into a normal (non-function) pointer.
//! * The **"actual"** type refers to the non-erased type.
//!
//! # Summary
//!
//! The types in this module are somewhat layered:
//!
//! * [`ErasedFnPtrPointee`] specifies the pointee type used for erased function
//!   pointers. It is defined to be [`c_void`][core::ffi::c_void] because that
//!   is what the dynamic loading APIs expect.
//! * [`RawErasedFnPtr`] is a raw pointer pointing to [`ErasedFnPtrPointee`]. It
//!   is only used when necessary for interacting with APIs exposed by other
//!   libraries.
//! * [`ErasedFnPtr`] conceptually wraps a [`RawErasedFnPtr`], but cannot be
//!   `null` (normal function pointers also cannot be `null`). It also
//!   implements [`Send`] and [`Sync`] (which aren't implemented for raw
//!   pointers).
//! * [`AtomicFnPtr`] conceptually wraps an [`ErasedFnPtr`], allowing it to be
//!   atomically updated.
//! * [`ErasedHandle`] stores a `'static` (leaked) reference to an
//!   [`AtomicFnPtr`].
//! * [`TypedHandle`] wraps an [`ErasedHandle`] and is the boundary between
//!   erased and non-erased (typed) layers.
//!
//! # Safety
//!
//! With the exception of [`ErasedFnPtrPointee`] and [`RawErasedFnPtr`] (which
//! are type aliases), all types defined in this module impose similar safety
//! requirements:
//!
//! 1) The **actual** type of the stored value must **never** change, and always
//!    be a function pointer implementing [`FnPtr`][crate::FnPtr].
//!    * This implies that stored values are always non-null.
//!
//! 2) With the exception of [`AtomicFnPtr`], the stored **value** must never
//!    change.
//!    * This makes it easier to reason about (1).
//!
//! # Code Style
//!
//! * Every `inner` field should be considered an "unsafe" field and will be
//!   marked as such once
//!   [RFC 3458](https://github.com/rust-lang/rust/issues/132922) is
//!   implemented.
//! * Methods should be `pub(super)` until they are needed outside this module.

pub use self::atomic::*;
pub use self::erased::*;
pub use self::fn_ptr::*;
pub use self::typed::*;

mod atomic;
mod erased;
mod fn_ptr;
mod typed;

#[cfg(test)]
mod tests;
