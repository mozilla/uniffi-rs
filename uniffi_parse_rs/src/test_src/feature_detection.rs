#[cfg(feature = "feature1")]
#[derive(uniffi::Record)]
pub struct Feature1 {
    #[cfg(feature = "feature2")]
    feature_2: bool,
}

#[derive(uniffi::Enum)]
#[cfg(not(feature = "feature1"))]
pub enum NotFeature1 {
    #[cfg(feature = "feature3")]
    Feature3,
}

#[derive(uniffi::Object)]
pub struct Object { }

#[cfg(feature = "feature1")]
#[uniffi::export]
impl Object {
    #[cfg(feature = "feature2")]
    pub fn feature_1_and_2(&self) {
    }
}

#[uniffi::export]
#[cfg(all(feature = "feature2", target_arch="x86_64"))]
pub fn feature2_and_x86_64() { }

#[cfg(any(feature = "feature2", target_arch="x86_64"))]
#[uniffi::export]
pub fn feature2_or_x86_64() { }

#[cfg_attr(
    any(
        all(feature = "feature2", not(target_arch="x86_64")),
        all(not(feature = "feature2"), target_arch = "x86_64"),
    ),
    uniffi::export
)]
pub fn feature2_xor_x86_64() { }

#[cfg_attr(feature = "feature1", derive(uniffi::Enum))]
#[cfg_attr(not(feature = "feature1"), derive(uniffi::Error))]
pub enum EnumOrError {}

#[cfg_attr(feature = "feature1", uniffi::export(name="renamed_feature1"))]
#[cfg_attr(not(feature = "feature1"), uniffi::export(name="renamed_no_feature1"))]
pub fn function_to_rename() { }
