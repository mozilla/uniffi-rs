cfg_select! {
    feature = "feature-a" => {
        #[derive(uniffi::Record)]
        pub struct RecordA;
    }
    feature = "feature-b" => {
        #[derive(uniffi::Record)]
        pub struct RecordB;
    }
    _ => {
        #[derive(uniffi::Record)]
        pub struct RecordC;
    }
}
