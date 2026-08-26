// Mirrors a binding crate that reuses custom types implemented by `custom_type_paths`:
// a local alias to the unparsed crate, plus `use_remote_type!` naming the implementing
// crate.  The alias target must resolve to the custom type registered over in
// `custom_type_paths`.

type Direct = external_crate2::Direct;

uniffi::use_remote_type!(custom_type_paths::Direct);

mod usage2 {
    use external_crate2::Direct;

    #[derive(uniffi::Record)]
    pub struct BindingRecord {
        pub direct: Direct,
    }
}
