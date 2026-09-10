// Mirrors a binding crate that reuses custom types implemented by `custom_type_paths`:
// it names the underlying type straight from the unparsed crate, and that must resolve
// to the custom type registered over in `custom_type_paths`.

mod usage2 {
    use external_crate2::Direct;

    #[derive(uniffi::Record)]
    pub struct BindingRecord {
        pub direct: Direct,
    }
}
