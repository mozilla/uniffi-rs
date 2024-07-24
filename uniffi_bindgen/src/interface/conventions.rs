use crate::{interface::Record, CodeOracle, ComponentInterface, Renameable};
use std::collections::{BTreeMap, BTreeSet};
use uniffi_meta::Type;

impl ComponentInterface {
    pub fn apply_naming_conventions<O: CodeOracle>(&mut self, oracle: O) {
        // Applying changes to the TypeUniverse
        for (_, t) in &mut self.types.type_definitions {
            if t.name().is_some() {
                t.rename(oracle.class_name(&t.name().unwrap()));
            }
        }

        let mut known: BTreeSet<Type> = BTreeSet::new();

        for t in &mut self.types.all_known_types.iter() {
            let mut ty = t.clone();
            if t.name().is_some() {
                ty.rename(oracle.class_name(&t.name().unwrap()));
            }
            known.insert(ty.clone());
        }

        self.types.all_known_types = known;

        // Conversions for CallbackInterfaceImpl.py
        for callback_interface in self.callback_interfaces.iter_mut() {
            // callback_interface.rename_display(oracle.fn_name(callback_interface.name()));

            for method in callback_interface.methods.iter_mut() {
                method.rename(oracle.fn_name(method.name()));

                for arg in method.arguments.iter_mut() {
                    arg.rename(oracle.var_name(arg.name()));
                }
            }
        }

        // Conversions for EnumTemplate.py and ErrorTemplate.py
        let errors = self.errors.clone();

        for enum_item in self.enums.values_mut() {
            if errors.contains(enum_item.name()) {
                enum_item.rename(oracle.class_name(enum_item.name()));

                for variant in &mut enum_item.variants {
                    variant.rename(oracle.class_name(variant.name()));

                    for field in &mut variant.fields {
                        field.rename(oracle.var_name(field.name()));
                    }
                }
            } else {
                enum_item.rename(oracle.enum_variant_name(enum_item.name()));

                for variant in &mut enum_item.variants {
                    variant.rename(oracle.enum_variant_name(variant.name()));
                    // The template
                    variant.set_is_name(oracle.var_name(variant.name()));

                    for field in &mut variant.fields {
                        field.rename(oracle.var_name(field.name()));
                    }
                }
            }
        }

        // Conversions for RecordTemplate.py
        let mut new_records: BTreeMap<String, Record> = BTreeMap::new();

        for (key, record_item) in self.records.iter_mut() {
            let mut record = record_item.clone();

            // We just want to prefix reserved keywords for the name, without modifying it
            record.rename(oracle.class_name(record_item.name()));

            for field in &mut record.fields {
                field.rename(oracle.var_name(field.name()));
            }

            // new_records.insert(oracle.class_name(key), record);
            new_records.insert(key.to_string(), record);
        }

        // One cannot alter a BTreeMap in place (with a few hacks maybe...), so we create a new one
        // with the adjusted names, and replace it.
        self.records = new_records;

        // Conversions for ObjectTemplate.py
        for object_item in self.objects.iter_mut() {
            for meth in &mut object_item.methods {
                meth.rename(oracle.fn_name(meth.name()));
            }

            for cons in &mut object_item.constructors {
                if !cons.is_primary_constructor() {
                    cons.rename(oracle.fn_name(cons.name()));
                }
            }
        }

        // Conversions for wrapper.py
        //TODO: Renaming the function name in wrapper.py is not currently tested
        //TODO: Renaming the callback_interface name in wrapper.py is currently not tested
        for func in self.functions.iter_mut() {
            func.rename(oracle.fn_name(func.name()));
        }

        for ci_def in self.callback_interfaces.iter_mut() {
            ci_def.rename_display(oracle.class_name(ci_def.name()));
        }
    }
}
