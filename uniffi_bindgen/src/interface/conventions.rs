use std::collections::BTreeSet;
use uniffi_meta::Type;
use crate::{CodeOracle, ComponentInterface, Renameable};
use crate::interface::FfiDefinition;

impl ComponentInterface {
    pub fn apply_naming_conventions<O: CodeOracle>(&mut self, oracle: O) {
        for (_, t) in &mut self.types.type_definitions {
            if t.name().is_some() {
                t.rename(oracle.var_name(&t.name().unwrap()))
            }
        }

        let mut known: BTreeSet<Type> = BTreeSet::new();
        dbg!("{:?}", self.clone());
        for t in &mut self.types.all_known_types.iter() {
            let mut ty = t.clone();
            if t.name().is_some() {
                ty.rename(oracle.var_name(&t.name().unwrap()))
            }
            known.insert(ty.clone());
        }

        self.types.all_known_types = known;
        dbg!("{:?}", self.clone());
        for function_item in self.functions.iter_mut() {
            function_item.rename(oracle.fn_name(function_item.name()));

            for arg in &mut function_item.arguments {
                arg.rename(oracle.var_name(arg.name()));
            }
        }

        for callback_interface in self.callback_interfaces.iter_mut() {
            callback_interface.rename_display(oracle.class_name(callback_interface.name()));
        }

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
                    variant.set_is_name(oracle.var_name(variant.name()));

                    for field in &mut variant.fields {
                        field.rename(oracle.var_name(field.name()));
                    }
                }
            }
        }
        //
        // for record_item in self.records.values_mut() {
        //     record_item.rename(oracle.class_name(record_item.name()));
        //
        //     for field in &mut record_item.fields {
        //         field.rename(oracle.var_name(field.name()));
        //     }
        // }
        //


        // for object_item in self.objects.iter_mut() {
        //     for meth in &mut object_item.methods {
        //         meth.rename(oracle.fn_name(meth.name()));
        //     }
        //
        //
        //     for (ffi_callback, m) in object_item.vtable_methods().iter_mut() {
        //         m.rename(oracle.fn_name(m.name()));
        //
        //         for arg in &mut ffi_callback.arguments {
        //             arg.rename(oracle.var_name(arg.name()));
        //         }
        //     }
        //
        //     for cons in &mut object_item.constructors {
        //         if !cons.is_primary_constructor() {
        //             cons.rename(oracle.fn_name(cons.name()));
        //         }
        //
        //     }
        // }

        for field in self.ffi_definitions() {
            match field {
                FfiDefinition::Function(mut ffi_function) => {
                    ffi_function.rename(oracle.var_name(ffi_function.name()));
                }
                FfiDefinition::CallbackFunction(mut callback_function) => {
                    callback_function.rename(oracle.var_name(callback_function.name()))
                }
                FfiDefinition::Struct(mut ffi_struct) => {
                    for f in &mut ffi_struct.fields {
                        f.rename(oracle.var_name(f.name()));
                    }
                }
            }
        }
    }
}
