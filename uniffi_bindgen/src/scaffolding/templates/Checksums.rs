{%- for (name, checksum) in ci.iter_checksums() %}
#[unsafe(no_mangle)]
#[doc(hidden)]
pub extern "C" fn r#{{ name }}() -> u16 {
    {{ checksum }}
}
{%- endfor %}
