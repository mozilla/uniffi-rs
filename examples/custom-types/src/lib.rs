// Custom Handle type which trivially wraps an i64.
pub struct Handle(pub i64);

// UniFFI must be able to convert each custom type into the builtin type and vice versa
impl From<i64> for Handle {
    fn from(value: i64) -> Self {
        Handle(value)
    }
}

impl From<Handle> for i64 {
    fn from(handle: Handle) -> Self {
        handle.0
    }
}

pub struct Url(pub url::Url);

// Convenience impl
impl From<url::Url> for Url {
    fn from(value: url::Url) -> Self {
        Url(value)
    }
}

// Used by UniFFI scaffolding
impl TryFrom<String> for Url {
    type Error = url::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(url::Url::parse(&value)?.into())
    }
}

impl From<Url> for String {
    fn from(url: Url) -> Self {
        url.0.into()
    }
}

// And a little struct and function that ties them together.
pub struct CustomTypesDemo {
    url: Url,
    handle: Handle,
}

pub fn get_custom_types_demo(v: Option<CustomTypesDemo>) -> CustomTypesDemo {
    v.unwrap_or_else(|| CustomTypesDemo {
        url: url::Url::parse("http://example.com/").unwrap().into(),
        handle: Handle(123),
    })
}

include!(concat!(env!("OUT_DIR"), "/custom-types.uniffi.rs"));
