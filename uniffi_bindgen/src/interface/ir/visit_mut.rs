/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;

use crate::interface::ir::*;

/// Visit all nodes of a BindingsIr
///
/// When `BindingsIr::visit_mut` is called, it walks the BindingsIr tree and calls one of these
/// methods for each node.  Each method has a default no-op implementation, so you only need to
/// implement the methods that matter.  The tree is walked depth-first, so all child nodes have
/// already been visited when one of the visit_* methods is called.
///
/// This is used for bindings IR specialization where the generalized BindingsIr is altered to be
/// specific to a particular bindings language.
pub trait VisitMut {
    fn visit_bindings_ir(&mut self, _bindings_ir: &mut BindingsIr) -> Result<()> {
        Ok(())
    }

    fn visit_record(&mut self, _record: &mut Record) -> Result<()> {
        Ok(())
    }

    fn visit_interface(&mut self, _interface: &mut Interface) -> Result<()> {
        Ok(())
    }

    fn visit_callback_interface(&mut self, _cbi: &mut CallbackInterface) -> Result<()> {
        Ok(())
    }

    fn visit_vtable(&mut self, _vtable: &mut VTable) -> Result<()> {
        Ok(())
    }

    fn visit_field(&mut self, _field: &mut Field) -> Result<()> {
        Ok(())
    }

    fn visit_enum(&mut self, _enum: &mut Enum) -> Result<()> {
        Ok(())
    }

    fn visit_custom_type(&mut self, _custom_type: &mut CustomType) -> Result<()> {
        Ok(())
    }

    fn visit_external_type(&mut self, _external_type: &mut ExternalType) -> Result<()> {
        Ok(())
    }

    fn visit_variant(&mut self, _variant: &mut Variant) -> Result<()> {
        Ok(())
    }

    fn visit_method(&mut self, _method: &mut Method) -> Result<()> {
        Ok(())
    }

    fn visit_uniffi_trait(&mut self, _trait: &mut UniffiTrait) -> Result<()> {
        Ok(())
    }

    fn visit_argument(&mut self, _argument: &mut Argument) -> Result<()> {
        Ok(())
    }

    fn visit_constructor(&mut self, _constructor: &mut Constructor) -> Result<()> {
        Ok(())
    }

    fn visit_function(&mut self, _function: &mut Function) -> Result<()> {
        Ok(())
    }

    fn visit_type(&mut self, _type: &mut Type) -> Result<()> {
        Ok(())
    }

    fn visit_return_type(&mut self, _type: &mut ReturnType) -> Result<()> {
        Ok(())
    }

    fn visit_throws_type(&mut self, _type: &mut ThrowsType) -> Result<()> {
        Ok(())
    }

    fn visit_literal(&mut self, _default: &mut Literal) -> Result<()> {
        Ok(())
    }

    fn visit_ffi_function(&mut self, _function: &mut FfiFunction) -> Result<()> {
        Ok(())
    }

    fn visit_ffi_function_type(&mut self, _function_type: &mut FfiFunctionType) -> Result<()> {
        Ok(())
    }

    fn visit_ffi_function_ref(&mut self, _function_ref: &mut FfiFunctionRef) -> Result<()> {
        Ok(())
    }

    fn visit_ffi_struct(&mut self, _struct: &mut FfiStruct) -> Result<()> {
        Ok(())
    }

    fn visit_ffi_field(&mut self, _field: &mut FfiField) -> Result<()> {
        Ok(())
    }

    fn visit_ffi_argument(&mut self, _argument: &mut FfiArgument) -> Result<()> {
        Ok(())
    }

    fn visit_ffi_type(&mut self, _ffi_type: &mut FfiType) -> Result<()> {
        Ok(())
    }

    fn visit_ffi_return_type(&mut self, _ffi_type: &mut FfiReturnType) -> Result<()> {
        Ok(())
    }
}

/// Trait for nodes to drive VisitMut
trait DriveVisitMut {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()>;
}

impl DriveVisitMut for BindingsIr {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.globals.visit_mut(visitor)?;
        self.checksum_checks.visit_mut(visitor)?;
        self.ffi_definitions.visit_mut(visitor)?;
        self.type_definitions.visit_mut(visitor)?;
        self.functions.visit_mut(visitor)?;
        visitor.visit_bindings_ir(self)?;
        Ok(())
    }
}

impl BindingsIr {
    // Also define visit_mut as an inherent method so users don't need to import the
    // `DriveVisitMut` trait.
    pub fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        <Self as DriveVisitMut>::visit_mut(self, visitor)
    }
}

impl DriveVisitMut for GlobalDefinitions {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ffi_rustbuffer_alloc.visit_mut(visitor)?;
        self.ffi_rustbuffer_free.visit_mut(visitor)?;
        self.ffi_rustbuffer_reserve.visit_mut(visitor)?;
        self.ffi_uniffi_contract_version.visit_mut(visitor)?;
        self.callback_interface_free_type.visit_mut(visitor)?;
        self.string_type.visit_mut(visitor)?;
        Ok(())
    }
}

impl DriveVisitMut for TypeDefinition {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        match self {
            TypeDefinition::Builtin(ty) => ty.visit_mut(visitor)?,
            TypeDefinition::Record(rec) => rec.visit_mut(visitor)?,
            TypeDefinition::Enum(en) => en.visit_mut(visitor)?,
            TypeDefinition::Interface(int) => int.visit_mut(visitor)?,
            TypeDefinition::CallbackInterface(cbi) => cbi.visit_mut(visitor)?,
            TypeDefinition::Custom(custom_type) => custom_type.visit_mut(visitor)?,
            TypeDefinition::External(external_type) => external_type.visit_mut(visitor)?,
        }
        Ok(())
    }
}

impl DriveVisitMut for Record {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.fields.visit_mut(visitor)?;
        self.self_type.visit_mut(visitor)?;
        visitor.visit_record(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Enum {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.discr_type.visit_mut(visitor)?;
        self.variants.visit_mut(visitor)?;
        self.self_type.visit_mut(visitor)?;
        visitor.visit_enum(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Interface {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ffi_clone.visit_mut(visitor)?;
        self.ffi_free.visit_mut(visitor)?;
        self.vtable.visit_mut(visitor)?;
        self.constructors.visit_mut(visitor)?;
        self.methods.visit_mut(visitor)?;
        self.self_type.visit_mut(visitor)?;
        self.uniffi_traits.visit_mut(visitor)?;
        visitor.visit_interface(self)?;
        Ok(())
    }
}

impl DriveVisitMut for CallbackInterface {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.methods.visit_mut(visitor)?;
        self.vtable.visit_mut(visitor)?;
        self.self_type.visit_mut(visitor)?;
        visitor.visit_callback_interface(self)?;
        Ok(())
    }
}

impl DriveVisitMut for CustomType {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.builtin.visit_mut(visitor)?;
        self.self_type.visit_mut(visitor)?;
        visitor.visit_custom_type(self)?;
        Ok(())
    }
}

impl DriveVisitMut for ExternalType {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.self_type.visit_mut(visitor)?;
        visitor.visit_external_type(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Variant {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.discr.visit_mut(visitor)?;
        self.fields.visit_mut(visitor)?;
        visitor.visit_variant(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Constructor {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.interface.visit_mut(visitor)?;
        self.arguments.visit_mut(visitor)?;
        self.return_type.visit_mut(visitor)?;
        self.throws_type.visit_mut(visitor)?;
        self.ffi_func.visit_mut(visitor)?;
        self.checksum_func.visit_mut(visitor)?;
        self.async_data.visit_mut(visitor)?;
        visitor.visit_constructor(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Function {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.arguments.visit_mut(visitor)?;
        self.return_type.visit_mut(visitor)?;
        self.throws_type.visit_mut(visitor)?;
        self.ffi_func.visit_mut(visitor)?;
        self.checksum_func.visit_mut(visitor)?;
        self.async_data.visit_mut(visitor)?;
        visitor.visit_function(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Method {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.interface.visit_mut(visitor)?;
        self.arguments.visit_mut(visitor)?;
        self.return_type.visit_mut(visitor)?;
        self.throws_type.visit_mut(visitor)?;
        self.ffi_func.visit_mut(visitor)?;
        self.checksum_func.visit_mut(visitor)?;
        self.async_data.visit_mut(visitor)?;
        visitor.visit_method(self)?;
        Ok(())
    }
}

impl DriveVisitMut for VTable {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ffi_type.visit_mut(visitor)?;
        self.ffi_init_callback.visit_mut(visitor)?;
        self.methods.visit_mut(visitor)?;
        self.method_ffi_types.visit_mut(visitor)?;
        visitor.visit_vtable(self)?;
        Ok(())
    }
}

impl DriveVisitMut for UniffiTrait {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        match self {
            Self::Debug { fmt } => fmt.visit_mut(visitor)?,
            Self::Display { fmt } => fmt.visit_mut(visitor)?,
            Self::Eq { eq, ne } => {
                eq.visit_mut(visitor)?;
                ne.visit_mut(visitor)?;
            }
            Self::Hash { hash } => hash.visit_mut(visitor)?,
        }
        visitor.visit_uniffi_trait(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Field {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ty.visit_mut(visitor)?;
        self.default.visit_mut(visitor)?;
        visitor.visit_field(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Argument {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ty.visit_mut(visitor)?;
        self.default.visit_mut(visitor)?;
        visitor.visit_argument(self)?;
        Ok(())
    }
}

impl DriveVisitMut for Type {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        match &mut self.kind {
            TypeKind::Custom { builtin, .. } => builtin.visit_mut(visitor)?,
            TypeKind::Optional { inner_type, .. } => inner_type.visit_mut(visitor)?,
            TypeKind::Sequence { inner_type, .. } => inner_type.visit_mut(visitor)?,
            TypeKind::Map {
                key_type,
                value_type,
                ..
            } => {
                key_type.visit_mut(visitor)?;
                value_type.visit_mut(visitor)?;
            }
            _ => (),
        }
        self.ffi_type.visit_mut(visitor)?;
        visitor.visit_type(self)?;
        Ok(())
    }
}

impl DriveVisitMut for ReturnType {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ty.visit_mut(visitor)?;
        visitor.visit_return_type(self)?;
        Ok(())
    }
}

impl DriveVisitMut for ThrowsType {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ty.visit_mut(visitor)?;
        visitor.visit_throws_type(self)?;
        Ok(())
    }
}

impl DriveVisitMut for AsyncData {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ffi_rust_future_poll.visit_mut(visitor)?;
        self.ffi_rust_future_complete.visit_mut(visitor)?;
        self.ffi_rust_future_free.visit_mut(visitor)?;
        self.foreign_future_result_type.visit_mut(visitor)?;
        Ok(())
    }
}

impl DriveVisitMut for Literal {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        match &mut self.kind {
            LiteralKind::UInt(_, _, ty)
            | LiteralKind::Int(_, _, ty)
            | LiteralKind::Float(_, ty)
            | LiteralKind::Enum(_, ty) => ty.visit_mut(visitor)?,
            LiteralKind::Some { inner } => inner.visit_mut(visitor)?,
            _ => (),
        }
        visitor.visit_literal(self)?;
        Ok(())
    }
}

impl DriveVisitMut for ChecksumCheck {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.func.visit_mut(visitor)?;
        Ok(())
    }
}

impl DriveVisitMut for FfiDefinition {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        match self {
            Self::Struct(s) => s.visit_mut(visitor)?,
            Self::Function(f) => f.visit_mut(visitor)?,
            Self::FunctionType(ft) => ft.visit_mut(visitor)?,
        };
        Ok(())
    }
}

impl DriveVisitMut for FfiStruct {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.fields.visit_mut(visitor)?;
        visitor.visit_ffi_struct(self)?;
        Ok(())
    }
}

impl DriveVisitMut for FfiFunction {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.arguments.visit_mut(visitor)?;
        self.return_type.visit_mut(visitor)?;
        visitor.visit_ffi_function(self)?;
        Ok(())
    }
}

impl DriveVisitMut for FfiFunctionType {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.arguments.visit_mut(visitor)?;
        self.return_type.visit_mut(visitor)?;
        visitor.visit_ffi_function_type(self)?;
        Ok(())
    }
}

impl DriveVisitMut for FfiField {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ty.visit_mut(visitor)?;
        visitor.visit_ffi_field(self)?;
        Ok(())
    }
}

impl DriveVisitMut for FfiArgument {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ty.visit_mut(visitor)?;
        visitor.visit_ffi_argument(self)?;
        Ok(())
    }
}

impl DriveVisitMut for FfiFunctionRef {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        visitor.visit_ffi_function_ref(self)?;
        Ok(())
    }
}

impl DriveVisitMut for FfiReturnType {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        self.ty.visit_mut(visitor)?;
        visitor.visit_ffi_return_type(self)?;
        Ok(())
    }
}

impl DriveVisitMut for FfiType {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        visitor.visit_ffi_type(self)?;
        Ok(())
    }
}

impl<T: DriveVisitMut> DriveVisitMut for Vec<T> {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        for child in self.iter_mut() {
            child.visit_mut(visitor)?;
        }
        Ok(())
    }
}

impl<T: DriveVisitMut> DriveVisitMut for Option<T> {
    fn visit_mut(&mut self, visitor: &mut impl VisitMut) -> Result<()> {
        if let Some(child) = self {
            child.visit_mut(visitor)?;
        }
        Ok(())
    }
}
