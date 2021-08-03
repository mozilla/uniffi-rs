use crate_one::CrateOneType;
use crate_two::CrateTwoType;

pub struct CombinedType {
    pub cot: CrateOneType,
    pub ctt: CrateTwoType,
}

fn get_combined_type(existing: Option<CombinedType>) -> CombinedType {
    existing.unwrap_or_else(|| CombinedType {
        cot: CrateOneType {
            sval: "hello".to_string(),
        },
        ctt: CrateTwoType { ival: 1 },
    })
}

include!(concat!(env!("OUT_DIR"), "/imported-types-lib.uniffi.rs"));
