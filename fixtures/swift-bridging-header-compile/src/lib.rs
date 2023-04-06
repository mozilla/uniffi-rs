pub fn no_arguments() {}

include!(concat!(
    env!("OUT_DIR"),
    "/swift-bridging-header-compile.uniffi.rs"
));
