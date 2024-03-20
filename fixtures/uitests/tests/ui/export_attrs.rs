fn main() {} /* empty main required by `trybuild` */

#[uniffi::constructor] // <--- would ideally be an error.
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

#[derive(uniffi::Record)]
// Records have explicit `#[uniffi()]` handling.
#[uniffi(flat_error)]
pub struct One {}

#[derive(uniffi::Record)]
pub struct Two {
    #[uniffi(flat_error)]
    inner: i32,
}

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum Error {
    #[error("Oops")]
    #[uniffi(flat_error)]
    Oops,
}

// ctor and method attribute confusion.
#[derive(uniffi::Object)]
struct OtherAttrs;

#[uniffi::export]
impl OtherAttrs {
    #[uniffi::constructor(foo = bar)]
    fn one() {}
}

#[uniffi::export]
impl OtherAttrs {
    #[uniffi::method(foo)]
    fn two() {}
}

// Defaults with the wrong argument names
#[uniffi::export(default(fooo = 0))]
pub fn func_with_default(foo: u32) -> u32 {
    foo
}

#[derive(uniffi::Object)]
struct ObjWithDefault(u32);

#[uniffi::export]
impl ObjWithDefault {
    #[uniffi::constructor(default(fooo = 0))]
    pub fn new(foo: u32) -> Self {
        Self(foo)
    }
}

#[uniffi::export]
impl ObjWithDefault {
    #[uniffi::method(default(fooo = 0))]
    pub fn foo_matches(&self, foo: u32) -> bool {
        self.0 == foo
    }
}


uniffi_macros::setup_scaffolding!();
