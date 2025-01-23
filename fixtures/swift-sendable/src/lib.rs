
#[derive(Debug, uniffi::Record)]
struct UniffiRecord {
    value: String,
}

#[derive(Debug, uniffi::Object)]
struct UniffiObject {
    value: String,
}

uniffi::setup_scaffolding!("swift_sendable");
