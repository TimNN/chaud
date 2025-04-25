use super::TypedHandle;
use super::dylib::Sym;
use super::handle::{ErasedFnPtr, ErasedHandle};
use super::registry::Registry;
use super::util::minilog;
use crate::FnPtr;
use foldhash::fast::FixedState;
use hashbrown::hash_map;
use parking_lot::{Mutex, RwLock};

/// A map of [`ErasedHandle`]s keyed by `K`, intended to be stored in a static.
///
/// [`FixedState`] is used because it has a `const` constructor. (And we don't
/// care about DOS resitance, since the keys aren't "attacker" controlled).
type HandleMap<K> = hashbrown::HashMap<K, ErasedHandle, FixedState>;

struct Handles {
    by_ptr: RwLock<HandleMap<ErasedFnPtr>>,
    by_sym: Mutex<HandleMap<Sym>>,
    registry: Mutex<Registry>,
}

static HANDLES: Handles = Handles {
    by_ptr: RwLock::new(HandleMap::with_hasher(FixedState::with_seed(0))),
    by_sym: Mutex::new(HandleMap::with_hasher(FixedState::with_seed(0))),
    registry: Mutex::new(Registry::new()),
};

#[inline]
pub fn create_handle<F: FnPtr>(sym: &'static str, f: F) -> TypedHandle<F> {
    let erased = ErasedFnPtr::erase(f);

    let handle = create_erased(sym, erased);

    // SAFETY: `create_erased` guaranteed that `handle` has the same actual type
    // as `erased`, which is `F`.
    unsafe { TypedHandle::<F>::new(handle) }
}

/// # Guarantees
///
/// That the returned value has the same actual type as `f`.
#[inline]
fn create_erased(sym: &'static str, f: ErasedFnPtr) -> ErasedHandle {
    {
        let by_ptr = HANDLES.by_ptr.read();

        if let Some(handle) = by_ptr.get(&f) {
            return *handle;
        }
    }

    create_slow(sym, f)
}

/// # Guarantees
///
/// That the returned value has the same actual type as `f`.
#[cold]
fn create_slow(sym: &'static str, f: ErasedFnPtr) -> ErasedHandle {
    minilog::init_once();

    let h = lookup_or_create_sym(sym, f);
    insert_by_ptr(f, h);
    h
}

/// # Guarantees
///
/// That the returned value has the same actual type as `f`.
fn lookup_or_create_sym(sym: &'static str, f: ErasedFnPtr) -> ErasedHandle {
    let sym = match Sym::new(sym) {
        Ok(sym) => sym,
        Err(e) => {
            log::error!("Invalid symbol, hot reloading will not work: {e:#}");
            // Create fallback handle.
            return ErasedHandle::new(f);
        }
    };

    match insert_or_get_by_sym(sym, f) {
        SymResult::Created(h) => {
            // Avoid the hot-reloading infrastructure in unit tests.
            if !cfg!(any(test, miri)) {
                HANDLES.registry.lock().register(sym, h);
            }
            h
        }
        SymResult::Found(h) => h,
    }
}

fn insert_by_ptr(f: ErasedFnPtr, h: ErasedHandle) {
    let mut by_ptr = HANDLES.by_ptr.write();

    match by_ptr.entry(f) {
        hash_map::Entry::Occupied(entry) => {
            if *entry.get() != h {
                log::error!("by_ptr concurrently updated with mismatched handle");
            }
        }
        hash_map::Entry::Vacant(entry) => {
            log::debug!("Registering handle for {f:?}: {h:?}");
            entry.insert(h);
        }
    }
}

enum SymResult {
    Created(ErasedHandle),
    Found(ErasedHandle),
}

/// # Guarantees
///
/// That the returned value (if any) has the same actual type as `f`.
fn insert_or_get_by_sym(sym: Sym, f: ErasedFnPtr) -> SymResult {
    let mut by_sym = HANDLES.by_sym.lock();

    match by_sym.entry_ref(&sym) {
        hash_map::EntryRef::Occupied(entry) => {
            // SAFETY: This assumes that a specific symbol always corresponds
            // to exactly one actual type. Violating this assumption is covered
            // under the `unsafe-hot-reload` feature opt-in.
            SymResult::Found(*entry.get())
        }
        hash_map::EntryRef::Vacant(entry) => {
            sym.check_syntax();

            let h = ErasedHandle::new(f);
            log::debug!("Registering handle for {sym:?}: {h:?}");
            entry.insert(h);
            SymResult::Created(h)
        }
    }
}
