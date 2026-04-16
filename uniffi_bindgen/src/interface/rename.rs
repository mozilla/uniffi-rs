/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
/// Implements renaming of items in the CI via toml configuration.
/// Intended to be called by bindings to update the local names of items
/// to be generated but not touching names in the ffi we need to use.
use crate::VisitMut;
use std::collections::HashMap;

pub fn rename(ci: &mut ComponentInterface, renames: &HashMap<String, toml::Table>) {
    let this_module_path = ci.crate_name().to_string();
    ci.visit_mut(&TomlRenamer {
        this_module_path,
        renames,
    })
}

struct TomlRenamer<'a> {
    this_module_path: String,
    renames: &'a HashMap<String, toml::Table>,
}

impl TomlRenamer<'_> {
    fn new_name(&self, module_path: &str, name: &str) -> Option<String> {
        let crate_name = module_path.split("::").next().unwrap();
        self.renames
            .get(crate_name)
            .and_then(|rename_table| rename_table.get(name))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

impl VisitMut for TomlRenamer<'_> {
    fn visit_record(&self, record: &mut Record) {
        let module_path = &record.module_path;
        let record_name = record.name().to_string();
        // fields
        for field in &mut record.fields {
            let field_path = format!("{}.{}", record_name, field.name);
            if let Some(new_name) = self.new_name(module_path, &field_path) {
                field.name = new_name;
            }
        }
        // the record type itself
        if let Some(new_name) = self.new_name(module_path, &record_name) {
            record.name = new_name;
        }
    }

    fn visit_object(&self, object: &mut Object) {
        let module_path = &object.module_path;
        if let Some(new_name) = self.new_name(module_path, object.name()) {
            object.name = new_name;
        }
    }

    fn visit_callback_interface(&self, iface: &mut CallbackInterface) {
        let module_path = &iface.module_path;
        if let Some(new_name) = self.new_name(module_path, &iface.name) {
            iface.name = new_name;
        }
    }

    fn visit_enum(&self, _is_error: bool, enum_: &mut Enum) {
        let module_path = &enum_.module_path;
        let enum_name = enum_.name().to_string();
        // enum variants
        for variant in &mut enum_.variants {
            let variant_name = variant.name.clone();
            let variant_path = format!("{}.{}", enum_name, variant_name);
            for field in &mut variant.fields {
                let field_path = format!("{}.{}", variant_path, field.name);
                if let Some(new_name) = self.new_name(module_path, &field_path) {
                    field.name = new_name;
                }
            }
            if let Some(new_name) = self.new_name(module_path, &variant_path) {
                variant.name = new_name;
            }
        }
        // the enum type itself
        if let Some(new_name) = self.new_name(module_path, &enum_name) {
            enum_.name = new_name;
        }
    }

    fn visit_type(&self, type_: &mut Type) {
        let module_path = type_.module_path().unwrap_or(&self.this_module_path);
        let crate_name = module_path.split("::").next().unwrap();
        let self_renames = self.renames.get(crate_name);
        type_.rename_recursive(&|name| {
            self_renames
                .and_then(|renames| renames.get(name))
                .and_then(|value| value.as_str())
                .unwrap_or(name)
                .to_string()
        });
    }

    fn visit_method(&self, object_name: &str, method: &mut Method) {
        let method_name = format!("{}.{}", object_name, method.name());
        // Rename the method
        if let Some(new_name) = self.new_name(&self.this_module_path, &method_name) {
            method.name = new_name;
        }
        // args
        for arg in &mut method.arguments {
            let arg_path = format!("{}.{}", method_name, arg.name);
            if let Some(new_name) = self.new_name(&self.this_module_path, &arg_path) {
                arg.name = new_name;
            }
        }
    }

    fn visit_constructor(&self, object_name: &str, constructor: &mut Constructor) {
        // ctor is the same as a method.
        let method_name = format!("{}.{}", object_name, constructor.name());
        if let Some(new_name) = self.new_name(&self.this_module_path, &method_name) {
            constructor.name = new_name;
        }
        for arg in &mut constructor.arguments {
            let arg_path = format!("{}.{}", method_name, arg.name);
            if let Some(new_name) = self.new_name(&self.this_module_path, &arg_path) {
                arg.name = new_name;
            }
        }
    }

    fn visit_function(&self, function: &mut Function) {
        let original_function_name = function.name.clone();
        if let Some(new_name) = self.new_name(&self.this_module_path, &function.name) {
            function.name = new_name;
        }
        // args
        for arg in &mut function.arguments {
            let arg_path = format!("{}.{}", original_function_name, arg.name);
            if let Some(new_name) = self.new_name(&self.this_module_path, &arg_path) {
                arg.name = new_name;
            }
        }
    }

    fn visit_error_name(&self, name: &mut String) {
        if let Some(new_name) = self.new_name(&self.this_module_path, name) {
            *name = new_name;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::{ComponentInterface, Enum, Function, Object, Record};
    use uniffi_meta::{
        EnumMetadata, EnumShape, FieldMetadata, FnMetadata, FnParamMetadata, ObjectImpl,
        ObjectMetadata, RecordMetadata, Type, VariantMetadata,
    };

    fn create_test_ci() -> ComponentInterface {
        let mut ci = ComponentInterface::new("test_crate");

        // Add test record with Option<OldEnum> field
        let record_meta = RecordMetadata {
            module_path: "test_crate".to_string(),
            name: "OldRecord".to_string(),
            remote: false,
            fields: vec![FieldMetadata {
                name: "field".to_string(),
                ty: Type::Optional {
                    inner_type: Box::new(Type::Enum {
                        module_path: "test_crate".to_string(),
                        name: "OldEnum".to_string(),
                    }),
                },
                default: None,
                docstring: None,
            }],
            docstring: None,
        };
        let record = Record::try_from(record_meta).unwrap();
        ci.add_record_definition(record).unwrap();

        // Add test object using metadata
        let object_meta = ObjectMetadata {
            module_path: "test_crate".to_string(),
            name: "OldObject".to_string(),
            imp: ObjectImpl::Struct,
            remote: false,
            docstring: None,
        };
        let object = Object::from(object_meta);
        ci.add_object_definition(object).unwrap();

        // Add test enum with Option<OldRecord> variant
        let enum_meta = EnumMetadata {
            module_path: "test_crate".to_string(),
            name: "OldEnum".to_string(),
            shape: EnumShape::Enum,
            discr_type: None,
            non_exhaustive: false,
            remote: false,
            variants: vec![VariantMetadata {
                name: "WithRecord".to_string(),
                fields: vec![FieldMetadata {
                    name: "record".to_string(),
                    ty: Type::Optional {
                        inner_type: Box::new(Type::Record {
                            module_path: "test_crate".to_string(),
                            name: "OldRecord".to_string(),
                        }),
                    },
                    default: None,
                    docstring: None,
                }],
                docstring: None,
                discr: None,
            }],
            docstring: None,
        };
        let enum_ = Enum::try_from(enum_meta).unwrap();
        ci.add_enum_definition(enum_).unwrap();

        // Add test function with Option<OldRecord> argument
        let function_meta = FnMetadata {
            module_path: "test_crate".to_string(),
            name: "old_function".to_string(),
            is_async: false,
            inputs: vec![FnParamMetadata {
                name: "arg".to_string(),
                ty: Type::Optional {
                    inner_type: Box::new(Type::Record {
                        module_path: "test_crate".to_string(),
                        name: "OldRecord".to_string(),
                    }),
                },
                by_ref: false,
                optional: false,
                default: None,
            }],
            return_type: None,
            throws: None,
            checksum: None,
            docstring: None,
        };
        let function = Function::from(function_meta);
        ci.add_function_definition(function).unwrap();

        ci
    }

    #[test]
    fn test_dot_notation_renaming() {
        let mut ci = create_test_ci();

        // Test dot notation for field and variant renaming
        let toml_str = r#"
        OldRecord = "NewRecord"
        "OldRecord.field" = "new_field"
        OldEnum = "NewEnum"
        "OldEnum.WithRecord" = "WithNewRecord"
        "OldEnum.WithRecord.record" = "new_record"
        old_function = "new_function"
        "old_function.arg" = "new_arg"
        "#;

        let renames: toml::Table = toml::from_str(toml_str).unwrap();
        let mut renames_map = HashMap::new();
        renames_map.insert("test_crate".to_string(), renames);
        rename(&mut ci, &renames_map);

        // Check that types were renamed
        assert_eq!(ci.record_definitions()[0].name(), "NewRecord");
        assert_eq!(ci.enum_definitions()[0].name(), "NewEnum");
        assert_eq!(ci.function_definitions()[0].name(), "new_function");

        // Check that field was renamed
        assert_eq!(ci.record_definitions()[0].fields()[0].name, "new_field");

        // Check that enum variant was renamed
        assert_eq!(ci.enum_definitions()[0].variants()[0].name, "WithNewRecord");

        // Check that variant field was renamed
        assert_eq!(
            ci.enum_definitions()[0].variants()[0].fields()[0].name,
            "new_record"
        );

        // Check that function argument was renamed
        assert_eq!(ci.function_definitions()[0].arguments()[0].name, "new_arg");
    }

    #[test]
    fn test_callback_interface_renaming() {
        use crate::interface::callbacks::CallbackInterface;
        use crate::interface::ffi::FfiFunction;

        let mut ci = ComponentInterface::new("test_crate");

        // Add a callback interface (trait)
        let callback_interface = CallbackInterface {
            name: "OldTrait".to_string(),
            module_path: "test_crate".to_string(),
            methods: vec![],
            docstring: None,
            ffi_init_callback: FfiFunction {
                name: "init".to_string(),
                arguments: vec![],
                return_type: None,
                is_async: false,
                has_rust_call_status_arg: true,
                is_object_free_function: false,
            },
        };
        ci.callback_interfaces.push(callback_interface);

        // Test callback interface renaming
        let toml_str = r#"
        OldTrait = "NewTrait"
        "#;

        let renames: toml::Table = toml::from_str(toml_str).unwrap();
        let mut renames_map = HashMap::new();
        renames_map.insert("test_crate".to_string(), renames);
        rename(&mut ci, &renames_map);

        // Check that callback interface was renamed
        assert_eq!(ci.callback_interfaces[0].name, "NewTrait");
    }
}
