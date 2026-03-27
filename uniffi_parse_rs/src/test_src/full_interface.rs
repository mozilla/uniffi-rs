//! Module docstring

/// Record docstring
#[derive(uniffi::Record)]
#[uniffi(name="RecordRenamed")]
pub struct Record {
    a: String,
    #[uniffi(name="b_renamed")]
    b: u32,
    #[uniffi(default)]
    c: Option<String>,
    #[uniffi(default = "test")]
    d: CustomType
}

mod submod {
    /// Enum docstring
    #[derive(uniffi::Enum)]
    #[uniffi(name="EnumRenamed")]
    #[non_exhaustive]
    pub enum Enum {
        /// Variant docstring
        One {
            r: super::Record,
        },
        /// Variant2 docstring
        #[uniffi(name="TwoRenamed")]
        Two(i8, u8),
    }
}

#[derive(uniffi::Enum)]
#[repr(u8)]
pub enum U8Enum {
    One,
    Two = 4,
    Three,
}

#[derive(uniffi::Enum)]
#[repr(i8)]
pub enum i8Enum {
    One,
    Two = -4,
    Three,
}

#[uniffi::export(default(a=0), name="func_renamed")]
/// Function docstring
pub fn func(a: u8, r: Record, e: &submod::Enum, s: &str) {
}

/// Object docstring
#[derive(uniffi::Object)]
#[uniffi(name="ObjectRenamed")]
pub struct Object;

#[derive(uniffi::Error)]
pub enum Error { }

#[derive(uniffi::Error)]
#[uniffi(flat_error)]
pub enum FlatError {
    // Since this is a flat error, we shouldn't ignore variant data
    Variant(std::io::Error),
    Variant2 {
        e: std::io::Error,
    }
}

pub type Result<T, E=Error> = std::result::Result<T, E>;

#[uniffi::export(name="ObjectRenamed")]
impl Object {
    /// Constructor docstring
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        todo!()
    }

    #[uniffi::constructor]
    pub fn failible_constructor(a: String) -> Result<Arc<Self>> {
        todo!()
    }

    #[uniffi::constructor(default(a="test"), name="constructor_with_default_renamed")]
    pub fn constructor_with_default(a: String) -> Result<Arc<Self>> {
        todo!()
    }

    /// Method docstring
    pub fn meth1(&self) -> Result<String, Error> {
        todo!()
    }

    #[uniffi::method(name="meth2_renamed")]
    pub fn meth2(&self, a: Arc<Self>) -> Result<Arc<Self>> {
        todo!()
    }

    #[uniffi::method(default(s))]
    pub fn meth3(self: Arc<Self>, s: String) -> Result<()> {
        todo!()
    }
}

#[uniffi::export]
/// Trait interface docstring
pub trait TraitInterface {
    fn method(&self) -> Result<u8> {
        todo!()
    }
}

#[uniffi::export(with_foreign)]
/// Trait interface (with foreign) docstring
pub trait TraitInterfaceWithForeign {
    fn method(&self, a: u8) -> Result<String> {
        todo!()
    }
}

#[uniffi::export(callback_interface)]
/// Callback interface docstring
pub trait CallbackInterface {
    fn method(&self, a: String) -> Result<Arc<Object>> {
        todo!()
    }
}

#[uniffi::export]
pub fn trait_fn(a: Arc<dyn TraitInterface>, b: &dyn TraitInterfaceWithForeign, c: Box<dyn CallbackInterface>) {
}

uniffi::custom_type!(CustomType, String, {
    into: |custom| custom.into(),
    try_from: |s| s.try_from(),
});
