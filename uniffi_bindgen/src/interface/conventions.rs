use crate::{ComponentInterface, VisitMut};

impl ComponentInterface {
    pub fn visit_mut<V: VisitMut>(&mut self, visitor: &V) {
        visitor.visit_record(self);
        visitor.visit_enum(self);
        visitor.visit_type(self);
        visitor.visit_object(self);
        visitor.visit_function(self);
        visitor.visit_callback_interface(self);
        visitor.visit_ffi_defitinion(self);
    }
}
