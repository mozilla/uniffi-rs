#[derive(Debug)]
pub struct ValidatedString(String);

impl From<ValidatedString> for String {
    fn from(value: ValidatedString) -> String {
        value.0
    }
}

impl TryFrom<String> for ValidatedString {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.contains("invalid") {
            Err(anyhow::anyhow!("Value cannot contain 'invalid'"))
        } else {
            Ok(ValidatedString(value))
        }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct RecordWithStrings {
    pub name: String,
    pub children: Vec<String>,
}

#[derive(Debug, uniffi::Record)]
pub struct RecordWithValidatedStrings {
    pub name: ValidatedString,
    pub children: Vec<ValidatedString>,
}

#[uniffi::export]
pub async fn takes_validated_strings(record: RecordWithValidatedStrings) {
    let _ = record;
}

uniffi::custom_type!(ValidatedString, String);

uniffi::setup_scaffolding!();

#[test]
fn test_string_validation_failure() {
    let base = RecordWithStrings {
        name: String::from("THIS_IS_A_RECORD"),
        children: vec![
            String::from("VALID"),
            String::from("This is invalid"),
        ]
    };

    let result: anyhow::Result<RecordWithValidatedStrings> = uniffi::Lift::<UniFfiTag>::try_lift(uniffi::Lower::<UniFfiTag>::lower(base));

    insta::assert_snapshot!(format!("{:?}", result.unwrap_err()));
}

#[test]
fn test_handle_failed_lift() {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;

    static CALLED: AtomicBool = AtomicBool::new(false);

    extern "C" fn callback(_: u64, _: uniffi::RustFuturePoll) {
        CALLED.store(true, Ordering::Relaxed);
    }

    let base = RecordWithStrings {
        name: String::from("this one is invalid this time"),
        children: vec![
            String::from("VALID"),
        ]
    };

    let lowered = uniffi::Lower::<UniFfiTag>::lower(base);

    let handle = uniffi_uniffi_fn_func_takes_validated_strings(lowered);

    // SAFETY: The remainder of these ffi calls are with a handle allocated by the
    // `uniffi_uniffi_fn_func_takes_validated_strings` function, which is for a Future
    // returning `()``

    unsafe {
        ffi_uniffi_rust_future_poll_void(
            handle,
            callback,
            0
        );
    }

    assert_eq!(CALLED.load(Ordering::Relaxed), true);

    let mut status = uniffi::RustCallStatus::default();

    unsafe {
        ffi_uniffi_rust_future_complete_void(handle, &mut status);
        ffi_uniffi_rust_future_free_void(handle);
    }

    assert!(status.code == uniffi::RustCallStatusCode::UnexpectedError);

    let ffi_error = <String as uniffi::Lift<UniFfiTag>>::try_lift(std::mem::ManuallyDrop::into_inner(status.error_buf)).unwrap();

    insta::assert_snapshot!(ffi_error);
}