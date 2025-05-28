// Code to re-export the UniFFI scaffolding functions.
#[allow(missing_docs)]
#[doc(hidden)]
pub const fn uniffi_reexport_hack() {}

#[doc(hidden)]
#[macro_export]
macro_rules! uniffi_reexport_scaffolding {
    () => {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn {{ ci.namespace() }}_uniffi_reexport_hack() {
            $crate::uniffi_reexport_hack()
        }
    };
}
