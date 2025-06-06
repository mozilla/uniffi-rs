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
