use crate::{ComponentInterface, VisitMut};

impl ComponentInterface {
    /// Walk down the [`ComponentInterface`] and adjust the names of each type
    /// based on the naming conventions of the supported languages.
    ///
    /// Each suppoerted language implements the [`VisitMut`] Trait and is able
    /// to alter the functions, enums etc. to its own naming conventions.
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
