use crate::{CodeOracle, ComponentInterface, Renameable};

impl ComponentInterface {
    pub fn apply_naming_conventions<O: CodeOracle>(&mut self, oracle: O) {
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

        for record_item in self.records.values_mut() {
            record_item.rename(oracle.class_name(record_item.name()));

            for field in &mut record_item.fields {
                field.rename(oracle.var_name(field.name()));
            }
        }

        for function_item in self.functions.iter_mut() {
            function_item.rename(oracle.fn_name(function_item.name()));

            for arg in &mut function_item.arguments {
                arg.rename(oracle.var_name(arg.name()));
            }
        }

        for f in self.function_definitions().iter_mut() {
            f.rename(oracle.fn_name(f.name()));
        }

        for object_item in self.objects.iter_mut() {
            object_item.rename(oracle.class_name(object_item.name()));

            for meth in &mut object_item.methods {
                meth.rename(oracle.fn_name(meth.name()));
            }

            for cons in &mut object_item.constructors {
                cons.rename(oracle.fn_name(cons.name()));
            }
        }

        for callback_interface in self.callback_interface_definitions().iter_mut() {
            callback_interface.rename(oracle.class_name(callback_interface.name()));
        }
    }
}
