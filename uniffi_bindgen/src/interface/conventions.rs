use crate::{interface::Record, CodeOracle, ComponentInterface, Renameable};
use std::collections::BTreeMap;

impl ComponentInterface {
    pub fn apply_naming_conventions<O: CodeOracle>(&mut self, oracle: O) {
        // Applying changes to the TypeUniverse
        // for (_, t) in &mut self.types.type_definitions {
        //
        //     if t.name().is_some() {
        //         t.rename(oracle.var_name(&t.name().unwrap()));
        //
        //     }
        // }
        //
        // let mut known: BTreeSet<Type> = BTreeSet::new();
        //
        // for t in &mut self.types.all_known_types.iter() {
        //     let mut ty = t.clone();
        //     if t.name().is_some() {
        //         ty.rename(oracle.external_types_name(&t.name().unwrap()));
        //     }
        //     known.insert(ty.clone());
        // }
        //
        // self.types.all_known_types = known;
        //
        // for function_item in self.functions.iter_mut() {
        //     function_item.rename(oracle.fn_name(function_item.name()));
        //
        //     for arg in &mut function_item.arguments {
        //         arg.rename(oracle.var_name(arg.name()));
        //     }
        // }

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
            record.rename(oracle.external_types_name(record_item.name()));

            for field in &mut record.fields {
                field.rename(oracle.var_name(field.name()));
            }

            // We just want to prefix reserved keywords for the name, without modifying it
            // new_records.insert(oracle.external_types_name(key), record);
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
        for func in self.functions.iter_mut() {
            func.rename(oracle.fn_name(func.name()));
        }
    }
}
