/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_namespace(input: general::Namespace, context: &Context) -> Result<Package> {
    let mut context = context.clone();
    context.update_from_namespace(&input);
    let config = context.config()?.clone();
    let has_async_fns = input.has_descendant(|callable: &general::Callable| callable.is_async());
    let imports = config
        .custom_types
        .values()
        .flat_map(|c| c.imports.clone())
        .flatten()
        .chain(has_async_fns.then(|| "kotlinx.coroutines.launch".into()))
        .collect();

    let interfaces = map_interfaces(&input.type_definitions, &context)?;
    let mut type_definitions = input.type_definitions.map_node(&context)?;
    add_uniffi_package_external_types(&mut type_definitions, &context)?;
    Ok(Package {
        name: context.package_name_for_namespace(&input.name)?.to_string(),
        crate_name: input.name,
        config,
        functions: input.functions.map_node(&context)?,
        interfaces,
        type_definitions,
        imports,
    })
}

/// Add TypeDefinition::External for FFI code in the `uniffi` package
fn add_uniffi_package_external_types(
    type_definitions: &mut Vec<TypeDefinition>,
    context: &Context,
) -> Result<()> {
    let mut simple_types_seen = HashSet::new();
    for type_def in type_definitions.iter_mut() {
        if let TypeDefinition::Simple(type_node) = type_def {
            simple_types_seen.insert(type_node.ty.clone());
        }
        if !type_def.type_node().ty.has_user_type() {
            *type_def = TypeDefinition::External(ExternalType {
                package_name: "uniffi".to_string(),
                self_type: type_def.type_node().clone(),
            });
        }
    }

    // Ensure we have a TypeDefinition::External for all builtin types.
    //
    // This is needed because the FFI code for complex types can depends on these.
    // For example `Option` depends on `readBoolean` and `writeBoolean`
    let builtin_types = context.builtin_types()?;
    builtin_types.visit(|type_node: &TypeNode| {
        if !simple_types_seen.contains(&type_node.ty) {
            type_definitions.push(TypeDefinition::External(ExternalType {
                package_name: "uniffi".to_string(),
                self_type: type_node.clone(),
            }));
        }
    });

    Ok(())
}

pub fn map_interfaces(
    type_defs: &[general::TypeDefinition],
    context: &Context,
) -> Result<Vec<Interface>> {
    type_defs
        .iter()
        .filter_map(|type_def| match type_def {
            general::TypeDefinition::Interface(int) => {
                Some(interfaces::map_interface(int, context))
            }
            general::TypeDefinition::CallbackInterface(cbi) => {
                Some(callbacks::interface_for_callback_interface(cbi, context))
            }
            _ => None,
        })
        .collect()
}

impl Package {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.crate_name)
    }

    pub fn jni_symbol_name(&self) -> String {
        self.name.replace("_", "_1").replace(".", "_")
    }

    pub fn jni_path(&self) -> String {
        self.name.replace(".", "/")
    }

    /// JNI methods to define/export from Rust
    ///
    /// Generates a list of (jni_method_name, callable) pairs.
    pub fn jni_methods(&self) -> impl Iterator<Item = (&str, JniMethodKind, &Callable)> {
        let functions = self.functions.iter().map(|f| {
            (
                f.jni_method_name.as_str(),
                JniMethodKind::Function,
                &f.callable,
            )
        });
        let methods = self.classes().flat_map(|c| {
            c.methods.iter().map(|m| {
                (
                    m.jni_method_name.as_str(),
                    JniMethodKind::Method,
                    &m.callable,
                )
            })
        });
        let constructors = self.classes().flat_map(|c| {
            c.constructors.iter().map(|c| {
                (
                    c.jni_method_name.as_str(),
                    JniMethodKind::Function,
                    &c.callable,
                )
            })
        });
        let trait_methods = self
            .uniffi_trait_methods()
            .flat_map(UniffiTraitMethods::jni_methods);

        functions
            .chain(methods)
            .chain(constructors)
            .chain(trait_methods)
    }

    pub fn classes(&self) -> impl Iterator<Item = &Class> {
        self.type_definitions
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDefinition::Class(cls) => Some(cls),
                _ => None,
            })
    }

    pub fn boxes(&self) -> impl Iterator<Item = &BoxedType> {
        self.type_definitions
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDefinition::Box(b) => Some(b),
                _ => None,
            })
    }

    pub fn uniffi_trait_methods(&self) -> impl Iterator<Item = &UniffiTraitMethods> {
        self.type_definitions
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDefinition::Class(c) => Some(&c.uniffi_trait_methods),
                TypeDefinition::Record(r) => Some(&r.uniffi_trait_methods),
                TypeDefinition::Enum(e) => Some(&e.uniffi_trait_methods),
                _ => None,
            })
    }

    pub fn rust_async_callable_results(&self) -> impl Iterator<Item = &CallableResult> {
        self.iter_callable_results_that_match(|c| c.is_for_rust_function() && c.is_async)
    }

    pub fn kotlin_sync_callable_results(&self) -> impl Iterator<Item = &CallableResult> {
        self.iter_callable_results_that_match(|c| c.is_for_kotlin_function() && !c.is_async)
    }

    pub fn kotlin_async_callable_results(&self) -> impl Iterator<Item = &CallableResult> {
        self.iter_callable_results_that_match(|c| c.is_for_kotlin_function() && c.is_async)
    }

    fn iter_callable_results_that_match(
        &self,
        filter: impl Fn(&Callable) -> bool,
    ) -> impl Iterator<Item = &CallableResult> {
        let mut seen = HashSet::new();
        let mut results = vec![];
        self.visit(|callable: &Callable| {
            if filter(callable) && seen.insert(callable.result.id()) {
                results.push(&callable.result);
            }
        });
        results.into_iter()
    }

    /// Types where we need to call the Kotlin lift function from Rust
    ///
    /// We need to do this for:
    ///  * non-primitive types that are returned
    ///  * error types that get thrown from Rust functions.
    ///  * the inner type of boxed types
    pub fn lift_kt_from_rust_types(&self) -> impl Iterator<Item = &TypeNode> {
        let mut type_nodes = IndexMap::new();

        self.visit(|callable: &Callable| {
            if let Some(return_type) = callable.return_type() {
                if !return_type.lowers_to_primitive() {
                    type_nodes.insert(return_type.id, return_type);
                }
            }
            if let Some(throws_type) = callable.throws_type() {
                type_nodes.insert(throws_type.id, throws_type);
            }
        });
        self.visit(|b: &BoxedType| {
            type_nodes.insert(b.inner.id, &b.inner);
        });

        type_nodes.into_values()
    }

    /// Static Rust variable to call the Kotlin lift function from Rust
    ///
    /// We create separate variables per package to avoid lookup errors when users generating a
    /// subset of the packages. When a scaffolding function needs to call one of the lift functions,
    /// it uses the one from the package where the function was defined.
    pub fn lift_kt_from_rust_var(&self, type_node: &TypeNode) -> String {
        format!(
            "UNIFFI_CACHED_LIFT_KT_{}_{}",
            self.crate_name.to_uppercase(),
            type_node.id
        )
    }
}
