use crate::{FnPtr, Func};

pub(crate) trait Sealed<T = ()> {}

macro_rules! one {
    ($($Arg:ident),*) => {
        impl<Ret, $($Arg),*> Sealed for fn($($Arg),*) -> Ret {}


        // SAFETY: `Self` is a function pointer.
        impl<Ret, $($Arg),*> FnPtr for fn($($Arg),*) -> Ret where Self: 'static {}

        impl<Ret, $($Arg,)* Item: Fn($($Arg),*) -> Ret> Sealed<fn($($Arg),*) -> Ret> for Item where fn($($Arg),*) -> Ret: FnPtr {}

        impl<Ret, $($Arg,)* Item: Fn($($Arg),*) -> Ret> Func<fn($($Arg),*) -> Ret> for Item where fn($($Arg),*) -> Ret: FnPtr {}
    }
}

one!();
one!(A);
one!(A, B);
one!(A, B, C);
one!(A, B, C, D);
one!(A, B, C, D, E);
one!(A, B, C, D, E, F);
one!(A, B, C, D, E, F, G);
one!(A, B, C, D, E, F, G, H);
one!(A, B, C, D, E, F, G, H, I);
