use ::cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature="feature-a")] {
        #[derive(uniffi::Record)]
        pub struct RecordA;
    } else if #[cfg(feature = "feature-b")] {
        #[derive(uniffi::Record)]
        pub struct RecordB;
    } else {
        #[derive(uniffi::Record)]
        pub struct RecordC;
    }
}
