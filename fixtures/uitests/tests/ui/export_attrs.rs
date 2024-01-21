fn main() {} /* empty main required by `trybuild` */

#[uniffi::constructor] // <--- should be an error!
// Someone might try to 'export' a struct instead of deriving it.
#[uniffi::export]
struct S {}

#[uniffi::export(Dixplay)]
struct S2 {}

#[derive(uniffi::Object)]
struct Object;

#[uniffi::export(callback_interface)]
impl Object {}

#[uniffi::export(with_foreign)]
fn foreign() {}

uniffi_macros::setup_scaffolding!();
