uniffi::include_scaffolding!("test");

pub struct CustomType(Vec<u8>);
impl crate::UniffiCustomTypeConverter for CustomType {
    type Builtin = Vec<u8>;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Self(val))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.0
    }
}

#[uniffi::export]
pub fn test_fn(_type_param: CustomType) {}
