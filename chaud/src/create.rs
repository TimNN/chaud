/// Generate a single `create*` function.
///
/// Call as, for example: `one!([unsafe] fn create2u(A, B));`.
macro_rules! one {
    ([$($kw:tt)*] fn $name:ident($($Arg:ident),*)) => {
        impl<Ret, $($Arg),*> crate::Handle<$($kw)* fn($($Arg),*) -> Ret>
        where $($kw)* fn($($Arg),*) -> Ret: crate::FnPtrBounds {
            /// See [Handle Creation][Self#creation] for details.
            #[inline]
            pub fn $name(f: $($kw)* fn($($Arg),*) -> Ret) -> Self {
                #[cfg(not(feature = "unsafe-hot-reload"))]
                return Self { repr: f };

                #[cfg(feature = "unsafe-hot-reload")]
                // SAFETY: `f` must be a function pointer, which it is.
                return Self { repr: unsafe { crate::hot::TypedHandle::create(f) } };
            }
        }
    }
}

/// Generate `create*` functions for 0-9 parameters.
///
/// Call as, for example: `many!([unsafe] [create0u, create1u, ..., create9u]);`
macro_rules! many {
    ([$($kw:tt)*] [$n0:ident, $n1:ident, $n2:ident, $n3:ident, $n4:ident, $n5:ident, $n6:ident, $n7:ident, $n8:ident, $n9:ident]) => {
        one!([$($kw)*] fn $n0());
        one!([$($kw)*] fn $n1(A));
        one!([$($kw)*] fn $n2(A, B));
        one!([$($kw)*] fn $n3(A, B, C));
        one!([$($kw)*] fn $n4(A, B, C, D));
        one!([$($kw)*] fn $n5(A, B, C, D, E));
        one!([$($kw)*] fn $n6(A, B, C, D, E, F));
        one!([$($kw)*] fn $n7(A, B, C, D, E, F, G));
        one!([$($kw)*] fn $n8(A, B, C, D, E, F, G, H));
        one!([$($kw)*] fn $n9(A, B, C, D, E, F, G, H, I));
    }
}

// This is in a module so that it is ordered before the items from the other
// `mod`s. (Which are in modules so that the cfgs show up properly in the docs).
mod impl_1 {
    many!([] [create0, create1, create2, create3, create4, create5, create6, create7, create8, create9]);
}

#[cfg(feature = "create-unsafe")]
mod impl_2 {
    many!([unsafe] [create0u, create1u, create2u, create3u, create4u, create5u, create6u, create7u, create8u, create9u]);
}

#[cfg(feature = "create-extern")]
mod impl_3 {
    many!([extern "C"] [create0e, create1e, create2e, create3e, create4e, create5e, create6e, create7e, create8e, create9e]);
}

#[cfg(all(feature = "create-unsafe", feature = "create-extern"))]
mod impl_4 {
    many!([unsafe extern "C"] [create0ue, create1ue, create2ue, create3ue, create4ue, create5ue, create6ue, create7ue, create8ue, create9ue]);
}
