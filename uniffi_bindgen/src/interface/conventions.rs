use crate::{CodeOracle, ComponentInterface, Renameable};

impl ComponentInterface {
    pub fn apply_naming_conventions<O: CodeOracle>(&mut self, oracle: O) {
        for enum_item in self.enums.values_mut() {
            enum_item.rename(oracle.class_name(enum_item.name()));
        }

        for record_item in self.records.values_mut() {
            record_item.rename(oracle.class_name(record_item.name()));
        }

        for function_item in self.functions.iter_mut() {
            function_item.rename(oracle.fn_name(function_item.name()));
        }

        for object_item in self.objects.iter_mut() {
            object_item.rename(oracle.class_name(object_item.name()));
        }

        for callback_interface in self.callback_interfaces.iter_mut() {
            callback_interface.rename(oracle.class_name(callback_interface.name()));
        }
    }
}