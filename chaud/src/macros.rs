#[doc(hidden)]
#[macro_export]
macro_rules! __internal_id {
    ($Handle:ident) => {
        concat!("CHAUD::", module_path!(), "::", stringify!($Handle))
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "unsafe-hot-reload")]
macro_rules! __internal_if_hot {
    ({$($hot:tt)*} $(else {$($cold:tt)*})?) => {
        $($hot)*
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "unsafe-hot-reload"))]
macro_rules! __internal_if_hot {
    ({$($hot:tt)*} $(else {$($cold:tt)*})?) => {
        $($($cold)*)?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_handle {
    (fn $func:ident $($t:tt)*) => { $crate::__internal_handle!(fn ($func) $($t)*); };
    (
        fn ($func:path)
        $(
            <$($lt:lifetime),* $(,)?>
        )?
        (
            $($arg:ident: $Arg:ty),* $(,)?
        )
        $(-> $Ret:ty)?
        as $Handle:ident
    ) => {
        #[derive(Copy, Clone)]
        pub struct $Handle {
            #[deprecated = "Construct via `Self::create()` instead."]
            __chaud_internal: ()
        }

        const _: () = {
            if !cfg!(test) && !cfg!(doc) && ::core::option_env!("CARGO_BIN_NAME").is_some() {
                ::core::panic!("Chaud handles must be defined in library crates");
            }

            $crate::__internal_if_hot!({
                #[unsafe(export_name = $crate::__internal_id!($Handle))]
                static HANDLE: $crate::__internal_hot::AtomicHandle<$Handle> =
                    $crate::__internal_hot::AtomicHandle::<$Handle>::new($func);

                static ID: $crate::__internal_hot::HandleId<$Handle> =
                    $crate::__internal_hot::HandleId::new(
                        $crate::__internal_id!($Handle),
                        &HANDLE,
                        |$($arg),*| HANDLE.get()($($arg),*)
                    );
            });

            // SAFETY: `Self::Ptr` is a function pointer.
            unsafe impl $crate::Handle for $Handle {
                type Ptr = $(for<$($lt),*>)? fn($($arg: $Arg),*) $(-> $Ret)?;

                #[inline]
                #[allow(deprecated)]
                fn create() -> Self {
                    $crate::__internal_if_hot!({ ID.register() });

                    Self { __chaud_internal: () }
                }

                #[inline]
                fn get(self) -> Self::Ptr {
                    $crate::__internal_if_hot!({ HANDLE.get() } else { $func })
                }

                fn __chaud_internal_do_not_implement() {}
            }

            impl $Handle {
                #[inline]
                pub fn create() -> Self {
                    <Self as $crate::Handle>::create()
                }

                #[inline]
                pub fn call$(<$($lt),*>)?(self $(, $arg: $Arg)*) $(-> $Ret)? {
                    <Self as $crate::Handle>::get(self)($($arg),*)
                }
            }

            impl ::core::fmt::Debug for $Handle {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(stringify!($Handle)).finish()
                }
            }
        };
    };
}
