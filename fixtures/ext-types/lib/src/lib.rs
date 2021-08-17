use ext_types_guid::Guid;
use uniffi_one::{Animal, IpAddr, UniffiOneType};
use wrapper_types::Handle;

fn get_uniffi_one_type(val: UniffiOneType) -> UniffiOneType {
    UniffiOneType { sval: format!("{} - {}", val.sval, val.sval) }
}

fn get_another_animal(animal: Animal) -> Animal {
    match animal {
        Animal::Dog => Animal::Cat,
        Animal::Cat => Animal::Dog,
    }
}

fn get_another_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4 { .. } => IpAddr::V6 { addr: "1:2:3:4".to_string() },
        IpAddr::V6 { .. } => IpAddr::V4 { q1: 127, q2: 0, q3: 0, q4: 1 },
    }
}

pub struct CombinedType {
    pub uot: UniffiOneType,
    pub uots: Vec<UniffiOneType>,
    pub maybe_uot: Option<UniffiOneType>,

    pub guid: Guid,
    pub guids: Vec<Guid>,
    pub maybe_guid: Option<Guid>,

    pub json: serde_json::Value,
    pub jsons: Vec<serde_json::Value>,
    pub maybe_json: Option<serde_json::Value>,

    pub handle: Handle,
    pub handles: Vec<Handle>,
    pub maybe_handle: Option<Handle>,

    pub animal: Animal,
    pub animals: Vec<Animal>,
    pub maybe_animal: Option<Animal>,
}

fn get_combined_type(existing: Option<CombinedType>) -> CombinedType {
    existing.unwrap_or_else(|| CombinedType {
        uot: UniffiOneType {
            sval: "hello".to_string(),
        },
        uots: vec![
            UniffiOneType {
                sval: "first of many".to_string(),
            },
            UniffiOneType {
                sval: "second of many".to_string(),
            },
        ],
        maybe_uot: None,

        guid: Guid("a-guid".into()),
        guids: vec![Guid("b-guid".into()), Guid("c-guid".into())],
        maybe_guid: None,

        json: serde_json::json!({"hello": "there"}),
        jsons: vec![],
        maybe_json: None,

        handle: Handle(123),
        handles: vec![Handle(1), Handle(2), Handle(3)],
        maybe_handle: Some(Handle(4)),

        animal: Animal::Dog,
        animals: vec![Animal::Cat, Animal::Dog],
        maybe_animal: Some(Animal::Cat),
    })
}

include!(concat!(env!("OUT_DIR"), "/ext-types-lib.uniffi.rs"));
