fn main() {} /* empty main required by `trybuild` */

#[derive(uniffi::Enum)]
pub enum AllSkipped {
    #[uniffi(skip)]
    Hidden(std::sync::mpsc::Receiver<()>),
}

uniffi_macros::setup_scaffolding!();
