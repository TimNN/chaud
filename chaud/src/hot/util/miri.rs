use parking_lot::Mutex;

pub trait Leakable: 'static + Send + Sync {}

impl<T: 'static + Send + Sync> Leakable for T {}

/// Mark the given allocation as intentionally leaked.
///
/// Sometimes Chaud intentionally leaks allocations. Miri complains about those
/// unless they are referenced by a `static` when the program ends.
///
/// While that will usually always be the case when running the entirety of
/// Chaud, it's not always the case for unit tests.
///
/// So to make unit tests simpler, we call this function when intentionally
/// leaking things, and put the allocation in a `static` so that it is ignored
/// by Miri.
pub fn intentionally_leaked(val: &'static impl Leakable) {
    static LEAKED: Mutex<Vec<&'static dyn Leakable>> = Mutex::new(vec![]);

    LEAKED.lock().push(val);
}
