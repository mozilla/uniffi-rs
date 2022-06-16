uniffi_callbacks::uniffi_reexport_scaffolding!();
uniffi_coverall::uniffi_reexport_scaffolding!();

#[cfg(test)]
mod tests {
    use cargo_metadata::Message;
    use libloading::{Library, Symbol};
    use std::ffi::CString;
    use std::os::raw::c_void;
    use std::process::{Command, Stdio};
    use uniffi::{FfiConverter, ForeignCallback, RustBuffer, RustCallStatus};
    use uniffi_bindgen::ComponentInterface;

    // Load the dynamic library that was built for this crate.  The external functions from
    // `uniffi_callbacks' and `uniffi_coverall` should be present.
    pub fn load_library() -> Library {
        let mut cmd = Command::new("cargo");
        cmd.arg("build").arg("--message-format=json").arg("--lib");
        cmd.stdout(Stdio::piped());
        let mut child = cmd.spawn().unwrap();
        let output = std::io::BufReader::new(child.stdout.take().unwrap());
        let artifacts = Message::parse_stream(output)
            .filter_map(|message| match message {
                Err(e) => panic!("{e}"),
                Ok(Message::CompilerArtifact(artifact)) => {
                    if artifact.target.name == "reexport_scaffolding_macro"
                        && artifact.target.kind.iter().any(|item| item == "cdylib")
                    {
                        Some(artifact)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if !child.wait().unwrap().success() {
            panic!("Failed to execute `cargo build`");
        }
        let artifact = match artifacts.len() {
            1 => &artifacts[0],
            n => panic!("Found {n} artfiacts from cargo build"),
        };
        let cdylib_files: Vec<_> = artifact
            .filenames
            .iter()
            .filter(|nm| matches!(nm.extension(), Some(std::env::consts::DLL_EXTENSION)))
            .collect();
        let library_path = match cdylib_files.len() {
            1 => cdylib_files[0].to_string(),
            _ => panic!("Failed to build exactly one cdylib file"),
        };
        unsafe { Library::new(library_path).unwrap() }
    }

    pub fn has_symbol<T>(library: &Library, name: &str) -> bool {
        unsafe {
            library
                .get::<T>(CString::new(name).unwrap().as_bytes_with_nul())
                .is_ok()
        }
    }

    pub fn get_symbol<'lib, T>(library: &'lib Library, name: &str) -> Symbol<'lib, T> {
        unsafe {
            library
                .get::<T>(CString::new(name).unwrap().as_bytes_with_nul())
                .unwrap()
        }
    }

    #[test]
    fn test_symbols_present() {
        let library = load_library();
        let coveralls_ci =
            ComponentInterface::from_webidl(include_str!("../../coverall/src/coverall.udl"))
                .unwrap();
        let callbacks_ci =
            ComponentInterface::from_webidl(include_str!("../../callbacks/src/callbacks.udl"))
                .unwrap();

        // UniFFI internal function
        assert!(has_symbol::<
            unsafe extern "C" fn(i32, &mut RustCallStatus) -> RustBuffer,
        >(
            &library, coveralls_ci.ffi_rustbuffer_alloc().name()
        ));

        // Top-level function
        assert!(
            has_symbol::<unsafe extern "C" fn(&mut RustCallStatus) -> u64>(
                &library,
                coveralls_ci
                    .get_function_definition("get_num_alive")
                    .unwrap()
                    .ffi_func()
                    .name()
            )
        );

        // Object method
        assert!(
            has_symbol::<unsafe extern "C" fn(&mut RustCallStatus) -> u64>(
                &library,
                coveralls_ci
                    .get_object_definition("Coveralls")
                    .unwrap()
                    .get_method("get_name")
                    .ffi_func()
                    .name()
            )
        );

        // Callback init func
        assert!(has_symbol::<
            unsafe extern "C" fn(ForeignCallback, &mut RustCallStatus) -> (),
        >(
            &library,
            callbacks_ci
                .get_callback_interface_definition("ForeignGetters")
                .unwrap()
                .ffi_init_callback()
                .name()
        ));
    }

    #[test]
    fn test_calls() {
        let mut call_status = RustCallStatus::default();
        let library = load_library();
        let coveralls_ci =
            ComponentInterface::from_webidl(include_str!("../../coverall/src/coverall.udl"))
                .unwrap();
        let object_def = coveralls_ci.get_object_definition("Coveralls").unwrap();

        let get_num_alive: Symbol<unsafe extern "C" fn(&mut RustCallStatus) -> u64> = get_symbol(
            &library,
            coveralls_ci
                .get_function_definition("get_num_alive")
                .unwrap()
                .ffi_func()
                .name(),
        );
        let coveralls_new: Symbol<
            unsafe extern "C" fn(RustBuffer, &mut RustCallStatus) -> *const c_void,
        > = get_symbol(
            &library,
            object_def.primary_constructor().unwrap().ffi_func().name(),
        );
        let coveralls_get_name: Symbol<
            unsafe extern "C" fn(*const c_void, &mut RustCallStatus) -> RustBuffer,
        > = get_symbol(
            &library,
            object_def.get_method("get_name").ffi_func().name(),
        );
        let coveralls_free: Symbol<unsafe extern "C" fn(*const c_void, &mut RustCallStatus) -> ()> =
            get_symbol(&library, object_def.ffi_object_free().name());

        let num_alive = unsafe { get_num_alive(&mut call_status) };
        assert_eq!(call_status.code, 0);
        assert_eq!(num_alive, 0);

        let obj_id = unsafe { coveralls_new(String::lower("TestName".into()), &mut call_status) };
        assert_eq!(call_status.code, 0);

        let name_buf = unsafe { coveralls_get_name(obj_id, &mut call_status) };
        assert_eq!(call_status.code, 0);
        assert_eq!(String::try_lift(name_buf).unwrap(), "TestName");

        let num_alive = unsafe { get_num_alive(&mut call_status) };
        assert_eq!(call_status.code, 0);
        assert_eq!(num_alive, 1);

        unsafe { coveralls_free(obj_id, &mut call_status) };
        assert_eq!(call_status.code, 0);

        let num_alive = unsafe { get_num_alive(&mut call_status) };
        assert_eq!(call_status.code, 0);
        assert_eq!(num_alive, 0);
    }
}
