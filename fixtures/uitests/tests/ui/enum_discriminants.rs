fn main() {} /* empty main required by `trybuild` */

#[repr(u64)]
#[derive(uniffi::Enum)]
pub enum EnumExpr {
    V = 1 + 1,
}

uniffi_macros::setup_scaffolding!();
