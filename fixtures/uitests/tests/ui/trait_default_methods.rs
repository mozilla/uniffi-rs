#[uniffi::export]
pub trait Trait: Send + Sync {
    // a method with a default impl causes problems (#2597 etc)
    fn default(&self) -> String {
        unreachable!()
    }
}

fn main() { /* empty main required by `trybuild` */}
