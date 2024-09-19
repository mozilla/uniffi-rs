use crate::interface::{Enum, FfiDefinition, Record};
use crate::{ComponentInterface, VisitMut};
use std::collections::{BTreeMap, BTreeSet};
use uniffi_meta::Type;

impl ComponentInterface {
    /// A generic interface for mutating items in the [`ComponentInterface`].
    ///
    /// Walk down the [`ComponentInterface`] and adjust the names of each type
    /// based on the naming conventions of the supported languages.
    ///
    /// Each suppoerted language implements the [`VisitMut`] Trait and is able
    /// to alter the functions, enums etc. to its own naming conventions.
    pub fn visit_mut<V: VisitMut>(&mut self, visitor: &V) {
        for type_ in self.types.type_definitions.values_mut() {
            visitor.visit_type(type_);
        }

        let mut all_known_types_altered: BTreeSet<Type> = BTreeSet::new();

        for type_ in self.types.all_known_types.iter() {
            let mut type_altered = type_.clone();
            visitor.visit_type(&mut type_altered);
            all_known_types_altered.insert(type_altered);
        }

        self.types.all_known_types = all_known_types_altered;

        let mut updated_enums: BTreeMap<String, Enum> = BTreeMap::new();
        let errors_clone = self.errors.clone();
        for (enum_name, enum_item) in self.enums.iter_mut() {
            let updated_key = visitor.visit_enum_key(&mut enum_name.clone());
            let is_error = errors_clone.contains(enum_item.name());

            visitor.visit_enum(is_error, enum_item);

            for variant in enum_item.variants.iter_mut() {
                visitor.visit_variant(is_error, variant);

                for field in variant.fields.iter_mut() {
                    visitor.visit_field(field);
                    visitor.visit_type(&mut field.type_);
                }
            }
            updated_enums.insert(updated_key, enum_item.clone());
        }
        self.enums = updated_enums;

        for record_item in self.records.values_mut() {
            visitor.visit_record(record_item);

            for field in &mut record_item.fields {
                visitor.visit_field(field);
                visitor.visit_type(&mut field.type_);
            }
        }
        self.fix_record_keys_after_rename();

        for function in self.functions.iter_mut() {
            visitor.visit_function(function);

            if function.clone().return_type.is_some() {
                let mut return_type = function.clone().return_type.unwrap();
                visitor.visit_type(&mut return_type);
                function.return_type = Some(return_type);
            }

            for argument in function.arguments.iter_mut() {
                visitor.visit_argument(argument);
                visitor.visit_type(&mut argument.type_);
            }
        }

        for object in self.objects.iter_mut() {
            visitor.visit_object(object);

            for method in object.methods.iter_mut() {
                visitor.visit_method(method);

                for argument in method.arguments.iter_mut() {
                    visitor.visit_argument(argument);
                    visitor.visit_type(&mut argument.type_);
                }

                if method.clone().return_type.is_some() {
                    let mut return_type = method.clone().return_type.unwrap();
                    visitor.visit_type(&mut return_type);
                    method.return_type = Some(return_type);
                }
            }

            for constructor in object.constructors.iter_mut() {
                visitor.visit_constructor(constructor);

                for argument in constructor.arguments.iter_mut() {
                    visitor.visit_argument(argument);
                    visitor.visit_type(&mut argument.type_);
                }
            }
        }

        for callback_interface in self.callback_interfaces.iter_mut() {
            for method in callback_interface.methods.iter_mut() {
                visitor.visit_method(method);

                for argument in method.arguments.iter_mut() {
                    visitor.visit_argument(argument);
                    visitor.visit_type(&mut argument.type_);
                }
            }

            for ffi_arg in callback_interface.ffi_init_callback.arguments.iter_mut() {
                visitor.visit_ffi_argument(ffi_arg);
            }
        }

        let mut throw_types_altered: BTreeSet<Type> = BTreeSet::new();

        for throw_type in self.callback_interface_throws_types.iter() {
            let mut type_ = throw_type.clone();
            visitor.visit_type(&mut type_);
            throw_types_altered.insert(type_);
        }

        self.callback_interface_throws_types = throw_types_altered;

        for ffi_definition in self.ffi_definitions() {
            if let FfiDefinition::Struct(mut ffi_struct) = ffi_definition {
                for field in ffi_struct.fields.iter_mut() {
                    visitor.visit_ffi_field(field);
                }
            }
        }

        self.errors = self
            .errors
            .drain()
            .map(|mut name| {
                visitor.visit_error_name(&mut name);
                name
            })
            .collect()
    }

    fn fix_record_keys_after_rename(&mut self) {
        let mut new_records: BTreeMap<String, Record> = BTreeMap::new();

        for record in self.records.values() {
            new_records.insert(record.name().to_string(), record.clone());
        }

        self.records = new_records;
    }
}
