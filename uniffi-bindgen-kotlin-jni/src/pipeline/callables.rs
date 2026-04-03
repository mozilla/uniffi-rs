/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_function(input: general::Function, context: &Context) -> Result<Function> {
    let module_path = context.normalize_rust_module_path(&input.module_path)?;
    let fully_qualified_name_rs = format!(
        "{module_path}::{}",
        names::escape_rust(&input.callable.orig_name)
    );
    Ok(Function {
        docstring: input.docstring,
        jni_method_name: format!(
            "function{}{}",
            context.current_crate_name()?.to_upper_camel_case(),
            input.callable.name.to_upper_camel_case()
        ),
        callable: map_callable(input.callable, fully_qualified_name_rs, context)?,
    })
}

pub fn map_method(input: general::Method, context: &Context) -> Result<Method> {
    let self_ty = match &input.callable.kind {
        general::CallableKind::Method { self_type, .. }
        | general::CallableKind::VTableMethod { self_type, .. } => self_type,
        _ => bail!("Invalid method callable kind: {:?}", input.callable.kind),
    };
    let Some(self_name) = self_ty.ty.name() else {
        bail!("Invalid Callable self type: {self_ty:?}")
    };
    let module_path = context.rust_module_path_for_type(&self_ty.ty)?;
    let fully_qualified_name_rs = format!(
        "{module_path}::{}::{}",
        names::escape_rust(self_name),
        names::escape_rust(&input.callable.orig_name)
    );
    let jni_method_name = format!(
        "method{}{}{}",
        context.current_crate_name()?.to_upper_camel_case(),
        self_name.to_upper_camel_case(),
        input.callable.name.to_upper_camel_case()
    );

    Ok(Method {
        docstring: input.docstring,
        jni_method_name,
        callable: map_callable(input.callable, fully_qualified_name_rs, context)?,
    })
}

pub fn map_constructor(input: general::Constructor, context: &Context) -> Result<Constructor> {
    let self_ty = match &input.callable.kind {
        general::CallableKind::Constructor { self_type, .. } => self_type,
        _ => bail!(
            "Invalid constructor callable kind: {:?}",
            input.callable.kind
        ),
    };
    let Some(self_name) = self_ty.ty.name() else {
        bail!(
            "Invalid constructor callable kind: {:?}",
            input.callable.kind
        );
    };
    let jni_method_name = format!(
        "constructor{}{}{}",
        context.current_crate_name()?.to_upper_camel_case(),
        self_name.to_upper_camel_case(),
        input.callable.name.to_upper_camel_case()
    );
    let module_path = context.rust_module_path_for_type(&self_ty.ty)?;
    let fully_qualified_name_rs = format!(
        "{module_path}::{}::{}",
        names::escape_rust(self_name),
        names::escape_rust(&input.callable.orig_name)
    );

    Ok(Constructor {
        docstring: input.docstring,
        jni_method_name,
        callable: map_callable(input.callable, fully_qualified_name_rs, context)?,
    })
}

fn map_callable(
    input: general::Callable,
    fully_qualified_name_rs: String,
    context: &Context,
) -> Result<Callable> {
    let kind = input.kind.map_node(context)?;
    let return_type = input.return_type.ty.map_node(context)?;
    let throws_type = input.throws_type.ty.map_node(context)?;
    let return_ffi = match &return_type {
        Some(type_node) if type_node.lowers_to_primitive() => ReturnFfi::Primitive {
            type_node: type_node.clone(),
            ffi_type: type_node.ffi_types[0],
        },
        Some(type_node) => ReturnFfi::Deconstruct {
            type_node: type_node.clone(),
            ffi_types: type_node.ffi_types.clone(),
        },
        None => ReturnFfi::Void,
    };

    let mut allocator = ArgAllocator::default();
    let receiver = match &kind {
        CallableKind::Method {
            self_type,
            takes_self_by_arc,
        } => Some(Argument {
            ty: self_type.clone(),
            by_ref: !takes_self_by_arc,
            ffi: argument_ffi(self_type, !takes_self_by_arc, &mut allocator),
            index: allocator.next_arg_index(),
            name: "".into(),
            optional: false,
        }),
        CallableKind::VTableMethod { self_type, .. } => Some(Argument {
            ty: self_type.clone(),
            by_ref: true,
            ffi: argument_ffi(self_type, true, &mut allocator),
            index: allocator.next_arg_index(),
            name: "".into(),
            optional: false,
        }),
        _ => None,
    };

    let arguments = map_arguments(input.arguments, &mut allocator, context)?;

    Ok(Callable {
        fully_qualified_name_rs,
        kind,
        is_async: input.async_data.is_some(),
        name: input.name,
        receiver,
        arguments,
        return_type,
        throws_type,
        return_ffi,
    })
}

fn map_arguments(
    input: Vec<general::Argument>,
    allocator: &mut ArgAllocator,
    context: &Context,
) -> Result<Vec<Argument>> {
    input
        .into_iter()
        .map(|arg| {
            let ty = arg.ty.map_node(context)?;

            Ok(Argument {
                name: arg.name,
                index: allocator.next_arg_index(),
                optional: arg.optional,
                ffi: argument_ffi(&ty, arg.by_ref, allocator),
                ty,
                by_ref: arg.by_ref,
            })
        })
        .collect()
}

fn argument_ffi(ty: &TypeNode, by_ref: bool, allocator: &mut ArgAllocator) -> ArgumentFfi {
    let id = ty.id;
    match (&ty.ty, by_ref) {
        (Type::Interface { .. }, true) => {
            ArgumentFfi::Custom {
                ffi_args: allocator.create_ffi_args(&[FfiType::Int64]),
                // Kotlin -> Rust uses optimized lower/lift functions that avoid a clone
                lower_fn_kt: format!("lowerObjectRef{id}"),
                lift_fn_rs: format!("lift_object_ref_{id}"),
                // Rust -> Kotlin use the normal functions, since there's no way to ensure that
                // Kotlin won't keep a reference to the argument.
                lift_fn_kt: ty.lift_fn_kt(),
                lower_fn_rs: ty.lower_fn_rs(),
            }
        }
        _ => ArgumentFfi::Standard {
            ffi_args: allocator.create_ffi_args(&ty.ffi_types),
        },
    }
}

impl Callable {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }

    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }

    pub fn has_receiver(&self) -> bool {
        self.receiver.is_some()
    }

    pub fn is_constructor(&self) -> bool {
        matches!(self.kind, CallableKind::Constructor { .. })
    }

    pub fn is_primary_constructor(&self) -> bool {
        matches!(self.kind, CallableKind::Constructor { primary: true, .. })
    }

    pub fn ffi_arguments(&self) -> impl Iterator<Item = &FfiArgument> {
        self.arguments.iter().flat_map(|a| a.ffi_args())
    }

    pub fn ffi_arguments_including_receiver(&self) -> impl Iterator<Item = &FfiArgument> {
        self.receiver
            .iter()
            .flat_map(|r| r.ffi_args())
            .chain(self.ffi_arguments())
    }

    pub fn return_type_kt(&self) -> &str {
        match &self.return_type {
            None => "Unit",
            Some(ty) => &ty.type_kt,
        }
    }

    pub fn arguments_including_receiver(&self) -> impl Iterator<Item = &Argument> {
        self.receiver.iter().chain(self.arguments.iter())
    }

    pub fn arg_list_kt(&self) -> String {
        self.arguments
            .iter()
            .map(|a| format!("{}: {}", a.name_kt(), a.ty.type_kt))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Argument {
    pub fn name_kt(&self) -> String {
        if self.name.is_empty() {
            "this".into()
        } else {
            format!("`{}`", self.name.to_lower_camel_case())
        }
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }

    pub fn lowers_to_primitive(&self) -> bool {
        self.ffi_args().len() == 1
    }

    pub fn ffi_args(&self) -> &[FfiArgument] {
        match &self.ffi {
            ArgumentFfi::Custom { ffi_args, .. } | ArgumentFfi::Standard { ffi_args } => ffi_args,
        }
    }

    // Function to lower the argument from Rust
    pub fn lower_fn_rs(&self) -> String {
        match &self.ffi {
            ArgumentFfi::Custom { lower_fn_rs, .. } => lower_fn_rs.clone(),
            ArgumentFfi::Standard { .. } => self.ty.lower_fn_rs(),
        }
    }

    // Function to lift the argument from Rust
    pub fn lift_fn_rs(&self) -> String {
        match &self.ffi {
            ArgumentFfi::Custom { lift_fn_rs, .. } => lift_fn_rs.clone(),
            ArgumentFfi::Standard { .. } => self.ty.lift_fn_rs(),
        }
    }

    // Function to lift the argument from Kotlin
    pub fn lift_fn_kt(&self) -> String {
        match &self.ffi {
            ArgumentFfi::Custom { lift_fn_kt, .. } => lift_fn_kt.clone(),
            ArgumentFfi::Standard { .. } => self.ty.lift_fn_kt(),
        }
    }

    // Function to lower the argument from Kotlin
    pub fn lower_fn_kt(&self) -> String {
        match &self.ffi {
            ArgumentFfi::Custom { lower_fn_kt, .. } => lower_fn_kt.clone(),
            ArgumentFfi::Standard { .. } => self.ty.lower_fn_kt(),
        }
    }
}

/// Generates FFI argument names and argument indexes for callables
#[derive(Default)]
struct ArgAllocator {
    arg_count: usize,
    ffi_arg_count: usize,
}

impl ArgAllocator {
    fn create_ffi_args(&mut self, ffi_types: &[FfiType]) -> Vec<FfiArgument> {
        ffi_types
            .iter()
            .map(|ffi_type| FfiArgument {
                name: self.next_ffi_arg_name(),
                ty: *ffi_type,
            })
            .collect()
    }

    fn next_ffi_arg_name(&mut self) -> String {
        let i = self.ffi_arg_count;
        self.ffi_arg_count += 1;
        format!("uniffi_arg_{i}")
    }

    fn next_arg_index(&mut self) -> usize {
        let i = self.arg_count;
        self.arg_count += 1;
        i
    }
}
