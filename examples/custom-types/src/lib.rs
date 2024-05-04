use url::Url;

// A custom guid defined via a proc-macro (ie, not referenced in the UDL)
pub struct ExampleCustomType(String);

// custom_newtype! is the easiest way to define custom types.
uniffi::custom_newtype!(ExampleCustomType, String);

// Custom Handle type which trivially wraps an i64.
pub struct Handle(pub i64);

// This one could also use custom_newtype!, but let's use Into and TryFrom instead
uniffi::custom_type!(Handle, i64);

// Defining `From<Handle> for i64` also gives us `Into<i64> for Handle`
impl From<Handle> for i64 {
    fn from(val: Handle) -> Self {
        val.0
    }
}

impl TryFrom<i64> for Handle {
    type Error = std::convert::Infallible;

    fn try_from(val: i64) -> Result<Handle, Self::Error> {
        Ok(Handle(val))
    }
}

// Custom TimeIntervalMs type which trivially wraps an i64.
pub struct TimeIntervalMs(pub i64);

// Another custom type, this time we will define an infallible conversion back to Rust.
uniffi::custom_type!(TimeIntervalMs, i64);

impl From<TimeIntervalMs> for i64 {
    fn from(val: TimeIntervalMs) -> Self {
        val.0
    }
}
// Defining `From<i64> for Handle` also gives us `Into<Handle> for i64`
impl From<i64> for TimeIntervalMs {
    fn from(val: i64) -> TimeIntervalMs {
        TimeIntervalMs(val)
    }
}

// Custom TimeIntervalSecDbl type which trivially wraps an f64.
pub struct TimeIntervalSecDbl(pub f64);

// custom_type! can take an additional parameter with closures to control the conversions
uniffi::custom_type!(TimeIntervalSecDbl, f64, {
    from_custom: |time_interval| time_interval.0,
    try_into_custom: |val| Ok(TimeIntervalSecDbl(val)),
});

// Custom TimeIntervalSecFlt type which trivially wraps an f32.
pub struct TimeIntervalSecFlt(pub f32);

// Let's go back to custom_newtype for this one.
uniffi::custom_newtype!(TimeIntervalSecFlt, f32);

// `Url` gets converted to a `String` to pass across the FFI.
// Use the `remote` param when types are defined in a different crate
uniffi::custom_type!(Url, String, {
    try_into_custom: |val| Ok(Url::parse(&val)?),
    from_custom: |obj| obj.into(),
    remote,
});

// And a little struct and function that ties them together.
pub struct CustomTypesDemo {
    url: Url,
    handle: Handle,
    time_interval_ms: TimeIntervalMs,
    time_interval_sec_dbl: TimeIntervalSecDbl,
    time_interval_sec_flt: TimeIntervalSecFlt,
}

pub fn get_custom_types_demo(v: Option<CustomTypesDemo>) -> CustomTypesDemo {
    v.unwrap_or_else(|| CustomTypesDemo {
        url: Url::parse("http://example.com/").unwrap(),
        handle: Handle(123),
        time_interval_ms: TimeIntervalMs(456000),
        time_interval_sec_dbl: TimeIntervalSecDbl(456.0),
        time_interval_sec_flt: TimeIntervalSecFlt(777.0),
    })
}

#[uniffi::export]
pub fn get_example_custom_type() -> ExampleCustomType {
    ExampleCustomType("abadidea".to_string())
}

uniffi::include_scaffolding!("custom-types");
