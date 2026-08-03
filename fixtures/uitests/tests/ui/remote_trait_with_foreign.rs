fn main() {} /* empty main required by `trybuild` */

// Foreign impls need UniFFI to add a hidden method to the trait, which is impossible for a
// trait defined in another crate, so `remote` and the foreign flags conflict.

#[uniffi::export(remote, foreign)]
pub trait RemoteForeign: Send + Sync {
    fn hello(&self) -> String;
}

#[uniffi::export(remote, rust, foreign)]
pub trait RemoteRustAndForeign: Send + Sync {
    fn hello(&self) -> String;
}

#[uniffi::export(remote, with_foreign)]
pub trait RemoteWithForeign: Send + Sync {
    fn hello(&self) -> String;
}

#[uniffi::export(remote, callback_interface)]
pub trait RemoteCallbackInterface: Send + Sync {
    fn hello(&self) -> String;
}

uniffi_macros::setup_scaffolding!();
