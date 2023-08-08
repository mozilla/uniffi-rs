use std::str::FromStr;

pub(crate) use http::HeaderMap;

/// A single HttpHeader for Uniffi bindings
pub(crate) struct HttpHeader {
    pub(crate) key: String,
    pub(crate) val: String,
}

/// Expose `http::HeaderMap` to Uniffi.
impl crate::UniffiCustomTypeConverter for http::HeaderMap {
    /// http::HeaderMap is a multimap so there may be multiple values
    /// per key. We represent this as a vector of `HttpHeader` (AKA
    /// `key` & `val`) where `key` may repeat.
    type Builtin = Vec<HttpHeader>;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(http::HeaderMap::from_iter(val.into_iter().filter_map(
            |h| {
                let n = http::HeaderName::from_str(&h.key).ok()?;
                let v = http::HeaderValue::from_str(&h.val).ok()?;
                Some((n, v))
            },
        )))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.iter()
            .map(|(k, v)| HttpHeader {
                key: k.as_str().to_string(),
                val: v.to_str().unwrap().to_string(),
            })
            .collect()
    }
}

pub fn get_headermap(v: String) -> HeaderMap {
    let n = http::HeaderName::from_str("test-header").unwrap();
    let v1 = http::HeaderValue::from_str("First value").unwrap();
    let v2 = http::HeaderValue::from_str(&v).unwrap();
    HeaderMap::from_iter([(n.clone(), v1), (n, v2)])
}

pub trait HeaderMapCallback {
    fn run(&self, arg: HeaderMap) -> HeaderMap;
}

pub fn run_callback(callback: Box<dyn HeaderMapCallback>) -> HeaderMap {
    let n = http::HeaderName::from_str("foo").unwrap();
    let v = http::HeaderValue::from_str("bar").unwrap();
    let h = HeaderMap::from_iter([(n, v)]);

    callback.run(h)
}

uniffi::include_scaffolding!("http_headermap");
