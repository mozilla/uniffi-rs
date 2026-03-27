pub struct Guid(u64);

#[derive(uniffi::Record)]
pub struct Record;

pub struct RecordWrapper(Record);
pub struct RecordWrapper2(Record);

uniffi::custom_newtype!(RecordWrapper2, r#Record);
uniffi::custom_type!(RecordWrapper, r#Record, {
    into: |wrapper| wrapper.0,
    try_from: |rec| Ok(RecordWrapper(rec)),
});
uniffi::custom_newtype!(r#Guid, r#u64);
