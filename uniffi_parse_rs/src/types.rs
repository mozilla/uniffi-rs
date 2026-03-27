/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    collections::{HashMap, HashSet},
    mem,
};

use proc_macro2::Span;
use syn::{
    ext::IdentExt, spanned::Spanned, AngleBracketedGenericArguments, GenericArgument, GenericParam,
    Generics, Ident, Path, PathArguments, TypeParamBound,
};

use crate::{
    attrs::TraitExportType, files::FileId, paths::LookupCache, BuiltinItem, Error, ErrorContext,
    ErrorKind::*, Ir, Item, RPath, Result,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ArgType {
    pub ty: uniffi_meta::Type,
    pub by_ref: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReturnType {
    pub ok: Option<uniffi_meta::Type>,
    pub err: Option<uniffi_meta::Type>,
}

pub struct SelfType {
    pub takes_self_by_arc: bool,
}

/// Type used in the exported interface
///
/// This is similar to `uniffi_meta::Type`, but it matches the Rust type system more closely.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    // Primitive types.
    Unit,
    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,
    Float32,
    Float64,
    Boolean,
    String,
    Str,
    SystemTime,
    Duration,
    // Types defined in the component API, each of which has a string name.
    Record {
        module_path: String,
        name: String,
    },
    Enum {
        module_path: String,
        name: String,
    },
    /// Type with `#[derive(uniffi::Object)]`
    Object {
        module_path: String,
        name: String,
    },
    /// Exported trait
    Trait {
        module_path: String,
        name: String,
        export_ty: TraitExportType,
    },
    // Structurally recursive types.
    Option(Box<Type>),
    Vec(Box<Type>),
    Arc(Box<Type>),
    Box(Box<Type>),
    Slice(Box<Type>),
    HashMap(Box<Type>, Box<Type>),
    Result(Box<Type>, Box<Type>),
    // Custom type on the scaffolding side
    Custom {
        module_path: String,
        name: String,
        builtin: Box<Type>,
    },
    Ref {
        mutable: bool,
        ty: Box<Type>,
    },
    SelfTy,
}

impl Type {
    pub fn is_self(&self) -> bool {
        matches!(self, Self::SelfTy)
    }

    fn try_into_uniffi_meta(
        self,
        source: FileId,
        span: Span,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<uniffi_meta::Type> {
        match self {
            Type::Unit => Err(Error::new(source, span, InvalidType)),
            Type::UInt8 => Ok(uniffi_meta::Type::UInt8),
            Type::Int8 => Ok(uniffi_meta::Type::Int8),
            Type::UInt16 => Ok(uniffi_meta::Type::UInt16),
            Type::Int16 => Ok(uniffi_meta::Type::Int16),
            Type::UInt32 => Ok(uniffi_meta::Type::UInt32),
            Type::Int32 => Ok(uniffi_meta::Type::Int32),
            Type::UInt64 => Ok(uniffi_meta::Type::UInt64),
            Type::Int64 => Ok(uniffi_meta::Type::Int64),
            Type::Float32 => Ok(uniffi_meta::Type::Float32),
            Type::Float64 => Ok(uniffi_meta::Type::Float64),
            Type::Boolean => Ok(uniffi_meta::Type::Boolean),
            Type::String => Ok(uniffi_meta::Type::String),
            Type::SystemTime => Ok(uniffi_meta::Type::Timestamp),
            Type::Duration => Ok(uniffi_meta::Type::Duration),
            Type::Record { module_path, name } => {
                Ok(uniffi_meta::Type::Record { module_path, name })
            }
            Type::Enum { module_path, name } => Ok(uniffi_meta::Type::Enum { module_path, name }),
            Type::Option(inner) => Ok(uniffi_meta::Type::Optional {
                inner_type: Box::new((*inner).try_into_uniffi_meta(source, span, self_ty)?),
            }),
            Type::Vec(inner) if matches!(*inner, Type::UInt8) => Ok(uniffi_meta::Type::Bytes),
            Type::Vec(inner) => Ok(uniffi_meta::Type::Sequence {
                inner_type: Box::new((*inner).try_into_uniffi_meta(source, span, self_ty)?),
            }),
            Type::HashMap(key, value) => Ok(uniffi_meta::Type::Map {
                key_type: Box::new((*key).try_into_uniffi_meta(source, span, self_ty)?),
                value_type: Box::new((*value).try_into_uniffi_meta(source, span, self_ty)?),
            }),
            Type::Object { module_path, name } => Ok(uniffi_meta::Type::Object {
                module_path,
                name,
                imp: uniffi_meta::ObjectImpl::Struct,
            }),
            Type::Arc(inner) => match *inner {
                Type::Object { module_path, name } => Ok(uniffi_meta::Type::Object {
                    module_path,
                    name,
                    imp: uniffi_meta::ObjectImpl::Struct,
                }),
                Type::Trait {
                    module_path,
                    name,
                    export_ty,
                } if export_ty.is_trait() => Ok(trait_to_uniffi_meta(module_path, name, export_ty)),
                Type::SelfTy => {
                    if let Some(self_ty) = self_ty {
                        Ok(self_ty.clone())
                    } else {
                        Err(Error::new(source, span, InvalidType))
                    }
                }
                _ => Err(Error::new(source, span, InvalidType)),
            },
            Type::Box(inner) => match *inner {
                Type::Trait {
                    module_path,
                    name,
                    export_ty: TraitExportType::CallbackInterface,
                } => Ok(uniffi_meta::Type::CallbackInterface { module_path, name }),
                ty => Ok(uniffi_meta::Type::Box {
                    inner_type: Box::new(ty.try_into_uniffi_meta(source, span, self_ty)?),
                }),
            },
            Type::Custom {
                module_path,
                name,
                builtin,
            } => Ok(uniffi_meta::Type::Custom {
                module_path,
                name,
                builtin: Box::new((*builtin).try_into_uniffi_meta(source, span, self_ty)?),
            }),
            Type::SelfTy => {
                if let Some(self_ty) = self_ty {
                    Ok(self_ty.clone())
                } else {
                    Err(Error::new(source, span, InvalidType))
                }
            }
            _ => Err(Error::new(source, span, InvalidType)),
        }
    }
}

fn trait_to_uniffi_meta(
    module_path: String,
    name: String,
    export_ty: TraitExportType,
) -> uniffi_meta::Type {
    match export_ty {
        TraitExportType::TraitInterface => uniffi_meta::Type::Object {
            module_path,
            name,
            imp: uniffi_meta::ObjectImpl::Trait,
        },
        TraitExportType::TraitInterfaceWithForeign => uniffi_meta::Type::Object {
            module_path,
            name,
            imp: uniffi_meta::ObjectImpl::CallbackTrait,
        },
        TraitExportType::CallbackInterface => {
            uniffi_meta::Type::CallbackInterface { module_path, name }
        }
    }
}

impl<'ir> RPath<'ir> {
    /// Resolve a `syn::Type` at this path into a `uniffi_meta::Type`
    pub fn resolve_uniffi_meta_type(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        ty: &syn::Type,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<uniffi_meta::Type> {
        self.resolve_type(ir, cache, ty)?
            .try_into_uniffi_meta(self.file_id(), ty.span(), self_ty)
    }

    fn resolve_type(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        ty: &syn::Type,
    ) -> Result<Type> {
        self._resolve_type(ir, cache, ty, &mut ResolveTypeContext::default())
            .map_err(|e| e.context(self.file_id(), ty.span(), "while resolving type"))
    }

    pub fn resolve_arg(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        syn_ty: &syn::Type,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<ArgType> {
        Ok(match self.resolve_type(ir, cache, syn_ty)? {
            Type::Ref { ty, .. } => match *ty {
                Type::Str => ArgType {
                    ty: uniffi_meta::Type::String,
                    by_ref: true,
                },
                Type::Trait {
                    module_path,
                    name,
                    export_ty,
                } => ArgType {
                    ty: trait_to_uniffi_meta(module_path, name, export_ty),
                    by_ref: true,
                },
                Type::Slice(inner) => ArgType {
                    ty: uniffi_meta::Type::Sequence {
                        inner_type: Box::new(inner.try_into_uniffi_meta(
                            self.file_id(),
                            syn_ty.span(),
                            self_ty,
                        )?),
                    },
                    by_ref: true,
                },
                ty => ArgType {
                    ty: ty.try_into_uniffi_meta(self.file_id(), syn_ty.span(), self_ty)?,
                    by_ref: true,
                },
            },
            ty => ArgType {
                ty: ty.try_into_uniffi_meta(self.file_id(), syn_ty.span(), self_ty)?,
                by_ref: false,
            },
        })
    }

    pub fn resolve_return_type(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        syn_ty: &syn::Type,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<ReturnType> {
        Ok(match self.resolve_type(ir, cache, syn_ty)? {
            Type::Unit => ReturnType {
                ok: None,
                err: None,
            },
            Type::Result(ok, err) => {
                let ok = match *ok {
                    Type::Unit => None,
                    ty => Some(ty.try_into_uniffi_meta(self.file_id(), syn_ty.span(), self_ty)?),
                };
                let err = err.try_into_uniffi_meta(self.file_id(), syn_ty.span(), self_ty)?;
                ReturnType { ok, err: Some(err) }
            }
            Type::Ref { .. } => {
                return Err(Error::new(self.file_id(), syn_ty.span(), InvalidReturnType));
            }
            ty => ReturnType {
                ok: Some(ty.try_into_uniffi_meta(self.file_id(), syn_ty.span(), self_ty)?),
                err: None,
            },
        })
    }

    pub fn resolve_self_type(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        syn_ty: &syn::Type,
    ) -> Result<SelfType> {
        match self.resolve_type(ir, cache, syn_ty)? {
            Type::Ref { mutable: false, ty } if ty.is_self() => Ok(SelfType {
                takes_self_by_arc: false,
            }),
            Type::Arc(ty) if ty.is_self() => Ok(SelfType {
                takes_self_by_arc: true,
            }),
            _ => Err(Error::new(self.file_id(), syn_ty.span(), InvalidSelfType)),
        }
    }

    fn _resolve_type(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        mut ty: &syn::Type,
        context: &mut ResolveTypeContext,
    ) -> Result<Type> {
        // Unwrap paren/groups
        loop {
            match ty {
                syn::Type::Paren(ty_paren) => {
                    ty = &ty_paren.elem;
                }
                syn::Type::Group(ty_group) => {
                    ty = &ty_group.elem;
                }
                _ => break,
            }
        }

        match ty {
            syn::Type::Reference(ty_ref) => Ok(Type::Ref {
                mutable: ty_ref.mutability.is_some(),
                ty: Box::new(self._resolve_type(ir, cache, &ty_ref.elem, context)?),
            }),
            syn::Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(Type::Unit),
            syn::Type::Slice(ty_slice) => Ok(Type::Slice(Box::new(self._resolve_type(
                ir,
                cache,
                &ty_slice.elem,
                context,
            )?))),
            syn::Type::Path(ty_path) => {
                if ty_path.path.is_ident("Self") {
                    return Ok(Type::SelfTy);
                }
                let path_to_type = self.resolve(ir, cache, &ty_path.path)?;
                // We can use `mem::take` to remove the generic_params and use them for this lookup.
                // If we need to recurse another level, we want a new set of generic params anyways.
                let generics = GenericArgs::new(
                    self.file_id(),
                    &ty_path.path,
                    mem::take(&mut context.generic_params),
                )?;

                match path_to_type.item()? {
                    Item::Record(rec) => {
                        generics.check_empty(self.file_id())?;
                        Ok(Type::Record {
                            module_path: path_to_type.parent_module()?.path_string(),
                            name: rec
                                .attrs
                                .name
                                .clone()
                                .unwrap_or_else(|| rec.ident.to_string()),
                        })
                    }
                    Item::Enum(en) => {
                        generics.check_empty(self.file_id())?;
                        Ok(Type::Enum {
                            module_path: path_to_type.parent_module()?.path_string(),
                            name: en
                                .attrs
                                .name
                                .clone()
                                .unwrap_or_else(|| en.ident.to_string()),
                        })
                    }
                    Item::Object(o) => {
                        generics.check_empty(self.file_id())?;
                        Ok(Type::Object {
                            module_path: path_to_type.parent_module()?.path_string(),
                            name: o.attrs.name.clone().unwrap_or_else(|| o.ident.to_string()),
                        })
                    }
                    Item::Type(type_alias) => {
                        if !context.paths_seen.insert(path_to_type.path_string()) {
                            return Err(Error::new(self.file_id(), ty.span(), CycleDetected));
                        }
                        let module_path = path_to_type.parent_module()?;
                        context.generic_params = generics
                            .resolve_generic_params(
                                ir,
                                cache,
                                &module_path,
                                self,
                                &type_alias.generics,
                            )
                            .context(self.file_id(), ty.span(), "while resolving type")?;
                        module_path._resolve_type(ir, cache, &type_alias.ty, context)
                    }
                    Item::Builtin(item) => {
                        if !item.has_generic_args() {
                            generics.check_empty(self.file_id())?;
                        }
                        Ok(match item {
                            BuiltinItem::UnitType => Type::Unit,
                            BuiltinItem::Boolean => Type::Boolean,
                            BuiltinItem::String => Type::String,
                            BuiltinItem::Str => Type::Str,
                            BuiltinItem::UInt8 => Type::UInt8,
                            BuiltinItem::Int8 => Type::Int8,
                            BuiltinItem::UInt16 => Type::UInt16,
                            BuiltinItem::Int16 => Type::Int16,
                            BuiltinItem::UInt32 => Type::UInt32,
                            BuiltinItem::Int32 => Type::Int32,
                            BuiltinItem::UInt64 => Type::UInt64,
                            BuiltinItem::Int64 => Type::Int64,
                            BuiltinItem::Float32 => Type::Float32,
                            BuiltinItem::Float64 => Type::Float64,
                            BuiltinItem::SystemTime => Type::SystemTime,
                            BuiltinItem::Duration => Type::Duration,
                            BuiltinItem::Option => {
                                let inner = generics.resolve1(ir, cache, self)?;
                                Type::Option(Box::new(inner))
                            }
                            BuiltinItem::Vec => {
                                let inner = generics.resolve1(ir, cache, self)?;
                                Type::Vec(Box::new(inner))
                            }
                            BuiltinItem::Arc => {
                                let inner = generics.resolve1(ir, cache, self)?;
                                Type::Arc(Box::new(inner))
                            }
                            BuiltinItem::Box => {
                                let inner = generics.resolve1(ir, cache, self)?;
                                Type::Box(Box::new(inner))
                            }
                            BuiltinItem::HashMap => {
                                let (key, value) = generics.resolve2(ir, cache, self)?;
                                Type::HashMap(Box::new(key), Box::new(value))
                            }
                            BuiltinItem::Result => {
                                let (ok, err) = generics.resolve2(ir, cache, self)?;
                                Type::Result(Box::new(ok), Box::new(err))
                            }
                            _ => return Err(Error::new(self.file_id(), ty.span(), InvalidType)),
                        })
                    }
                    Item::Trait { .. } => {
                        Err(Error::new(self.file_id(), ty.span(), TraitWithoutDyn))
                    }
                    Item::CustomType(custom_type) => {
                        let module_path = path_to_type.parent_module()?;
                        let builtin =
                            module_path._resolve_type(ir, cache, &custom_type.builtin, context)?;
                        Ok(Type::Custom {
                            module_path: module_path.path_string(),
                            name: custom_type.ident.unraw().to_string(),
                            builtin: Box::new(builtin),
                        })
                    }
                    _ => Err(Error::new(self.file_id(), ty.span(), InvalidType)),
                }
            }
            syn::Type::TraitObject(ty_trait) => {
                let trait_bounds = ty_trait
                    .bounds
                    .iter()
                    .filter_map(|param_bound| match param_bound {
                        TypeParamBound::Trait(trait_bound) => Some(&trait_bound.path),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if trait_bounds.len() != 1 {
                    return Err(Error::new(self.file_id(), ty.span(), InvalidDynTrait));
                }
                let trait_path = self.resolve(ir, cache, trait_bounds[0])?;
                match trait_path.item()? {
                    Item::Trait(tr) => Ok(Type::Trait {
                        module_path: self.path_string(),
                        name: tr.ident.unraw().to_string(),
                        export_ty: tr.attrs.export_ty,
                    }),
                    _ => Err(Error::new(self.file_id(), ty.span(), ExpectedTrait)),
                }
            }
            _ => Err(Error::new(self.file_id(), ty.span(), InvalidType)),
        }
    }
}

/// Context for `resolve_type` this tracks data needed for recursive calls via type aliases
#[derive(Default)]
struct ResolveTypeContext {
    /// Paths that we've seen while resolving a type.
    ///
    /// Used to detect cycles between type aliases
    paths_seen: HashSet<String>,
    /// Context for looking up generic arguments
    ///
    /// This stores the generic parameters from the previous type aliases.
    /// For example, `T`, `E` from `type ResultAlias<T, E=Error> = Result<T, E>`
    generic_params: HashMap<Ident, Type>,
}

struct GenericArgs<'a> {
    args: Option<&'a AngleBracketedGenericArguments>,
    generic_params: HashMap<Ident, Type>,
}

impl<'a> GenericArgs<'a> {
    fn new(source: FileId, path: &'a Path, generic_params: HashMap<Ident, Type>) -> Result<Self> {
        for seg in path.segments.iter().take(path.segments.len() - 1) {
            if !seg.arguments.is_empty() {
                return Err(Error::new(source, seg.span(), InvalidGenericArg));
            }
        }
        let last_segment = &path.segments.last().unwrap();
        match &last_segment.arguments {
            PathArguments::Parenthesized(_) => {
                Err(Error::new(source, last_segment.span(), InvalidGenericArg))
            }
            PathArguments::AngleBracketed(args) if !args.args.is_empty() => Ok(Self {
                args: Some(args),
                generic_params,
            }),
            _ => Ok(Self {
                args: None,
                generic_params,
            }),
        }
    }

    fn check_empty(&self, source: FileId) -> Result<()> {
        match &self.args {
            Some(_) => Err(Error::new(source, self.args.span(), InvalidGenericArg)),
            None => Ok(()),
        }
    }

    fn resolve1<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
    ) -> Result<Type> {
        let args = self.get_args_and_check_len(path.file_id(), 1)?;
        self.resolve_arg(ir, cache, path, args[0])
    }

    fn resolve2<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
    ) -> Result<(Type, Type)> {
        let args = self.get_args_and_check_len(path.file_id(), 2)?;
        Ok((
            self.resolve_arg(ir, cache, path, args[0])?,
            self.resolve_arg(ir, cache, path, args[1])?,
        ))
    }

    fn args(&self) -> Vec<&GenericArgument> {
        match &self.args {
            None => vec![],
            Some(args) => args.args.iter().collect(),
        }
    }

    fn get_args_and_check_len(
        &self,
        source: FileId,
        expected: usize,
    ) -> Result<Vec<&GenericArgument>> {
        let args = self.args();
        if args.len() != expected {
            Err(Error::new(
                source,
                self.args.span(),
                InvalidGenericArgLength { expected },
            ))
        } else {
            Ok(args)
        }
    }

    fn resolve_generic_params<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        generic_mod: &RPath<'ir>,
        type_mod: &RPath<'ir>,
        generics: &Generics,
    ) -> Result<HashMap<Ident, Type>> {
        let type_params = generics
            .params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Type(t) => Some(t),
                GenericParam::Lifetime(_) | GenericParam::Const(_) => None,
            })
            .collect::<Vec<_>>();
        let args = self.args();
        if args.len() > type_params.len() {
            return Err(Error::new(
                generic_mod.file_id(),
                self.args.span(),
                TooManyGenericArgs,
            ));
        }
        let mut resolved = HashMap::new();
        for (i, param) in type_params.iter().enumerate() {
            if i < args.len() {
                resolved.insert(
                    param.ident.clone(),
                    self.resolve_arg(ir, cache, type_mod, args[i])?,
                );
            } else if let Some(default_ty) = &param.default {
                resolved.insert(
                    param.ident.clone(),
                    generic_mod._resolve_type(
                        ir,
                        cache,
                        default_ty,
                        &mut ResolveTypeContext::default(),
                    )?,
                );
            } else {
                return Err(Error::new(
                    generic_mod.file_id(),
                    self.args.span(),
                    MissingGenericArg,
                ));
            }
        }
        Ok(resolved)
    }

    fn resolve_arg<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
        arg: &GenericArgument,
    ) -> Result<Type> {
        match arg {
            GenericArgument::Type(ty) => {
                // Check if the type matches an identifier in the type_context (i.e. it matches a
                // generic param the type alias that we're resolving).
                match ty {
                    syn::Type::Path(ty_path)
                        if ty_path.path.leading_colon.is_none()
                            && ty_path.path.segments.len() == 1 =>
                    {
                        if let Some(resolved_type) = ty_path
                            .path
                            .get_ident()
                            .and_then(|i| self.generic_params.get(i))
                        {
                            return Ok(resolved_type.clone());
                        }
                    }
                    _ => (),
                }
                path.resolve_type(ir, cache, ty)
            }
            _ => Err(Error::new(
                path.file_id(),
                self.args.span(),
                MissingGenericArg,
            )),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{paths::tests::path_for_module, ErrorKind};
    use uniffi_meta::ObjectImpl;

    fn run_resolve_type<'ir>(
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &str,
        ty: &str,
    ) -> Result<Type, ErrorKind> {
        let module = path_for_module(ir, module_path);
        module
            .resolve_type(ir, cache, &syn::parse_str(ty).unwrap())
            .map_err(|e| e.kind)
    }

    fn run_resolve_uniffi_meta_type<'ir>(
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &str,
        ty: &str,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<uniffi_meta::Type, ErrorKind> {
        let module = path_for_module(ir, module_path);
        module
            .resolve_uniffi_meta_type(ir, cache, &syn::parse_str(ty).unwrap(), self_ty)
            .map_err(|e| e.kind)
    }

    fn run_resolve_arg<'ir>(
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &str,
        ty: &str,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<ArgType, ErrorKind> {
        let module = path_for_module(ir, module_path);
        module
            .resolve_arg(ir, cache, &syn::parse_str(ty).unwrap(), self_ty)
            .map_err(|e| e.kind)
    }

    #[test]
    fn test_resolve_builtin_types() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "()"),
            Ok(Type::Unit)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "String"),
            Ok(Type::String)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "bool"),
            Ok(Type::Boolean)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "i8"),
            Ok(Type::Int8)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "std::primitive::u8"),
            Ok(Type::UInt8)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "::std::primitive::i16"),
            Ok(Type::Int16)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "::core::primitive::u16"),
            Ok(Type::UInt16)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "i32"),
            Ok(Type::Int32)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "u32"),
            Ok(Type::UInt32)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "i64"),
            Ok(Type::Int64)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "u64"),
            Ok(Type::UInt64)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "f32"),
            Ok(Type::Float32)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "f64"),
            Ok(Type::Float64)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "f64"),
            Ok(Type::Float64)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "std::time::SystemTime"),
            Ok(Type::SystemTime)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "std::time::Duration"),
            Ok(Type::Duration)
        );
        // Builtin type using an alias
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "RenamedU64"),
            Ok(Type::UInt64)
        );
        // Generics are invalid for simple types
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "String<u32>"),
            Err(ErrorKind::InvalidGenericArg)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "String<>"),
            Ok(Type::String)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "std::primitive::unit"),
            Ok(Type::Unit)
        );
    }

    #[test]
    fn test_resolve_user_types() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "String"),
            Ok(Type::String)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "std::primitive::u32"),
            Ok(Type::UInt32)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "Mod1Record"),
            Ok(Type::Record {
                module_path: "types::mod1".into(),
                name: "Mod1Record".into(),
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "TestEnum"),
            Ok(Type::Enum {
                module_path: "types".into(),
                name: "TestEnum".into(),
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "TestEnum"),
            Ok(Type::Enum {
                module_path: "types".into(),
                name: "TestEnum".into(),
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "TestError"),
            Ok(Type::Enum {
                module_path: "types".into(),
                name: "TestError".into(),
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "TestInterface"),
            Ok(Type::Object {
                module_path: "types".into(),
                name: "TestInterface".into(),
            })
        );
        // Generics are invalid for user types
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "TestRecord<String>"),
            Err(ErrorKind::InvalidGenericArg)
        );
    }

    #[test]
    fn test_resolve_custom_types() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "JsonObject"),
            Ok(Type::Custom {
                module_path: "types".into(),
                name: "JsonObject".into(),
                builtin: Box::new(Type::String),
            })
        );
        // Builtin is a user type and also that type is not reachable from the
        // module where the custom type is used.
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "super::CustomRecord"),
            Ok(Type::Custom {
                module_path: "types".into(),
                name: "CustomRecord".into(),
                builtin: Box::new(Type::Record {
                    module_path: "types".into(),
                    name: "TestRecord".into(),
                }),
            })
        );

        // Test a custom_newtype
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "Guid"),
            Ok(Type::Custom {
                module_path: "types".into(),
                name: "Guid".into(),
                builtin: Box::new(Type::UInt64),
            })
        );

        // Test complex imports.  Here the `uniffi::custom_type` macro has been imported through
        // multiple use statements and also renamed.
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "Handle"),
            Ok(Type::Custom {
                module_path: "types::mod1".into(),
                name: "Handle".into(),
                builtin: Box::new(Type::UInt64),
            })
        );
    }

    #[test]
    fn test_resolve_compound_types() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "Vec<String>"),
            Ok(Type::Vec(Box::new(Type::String)))
        );
        assert_eq!(
            run_resolve_type(
                &ir,
                &mut cache,
                "types",
                "std::collections::HashMap<u32, TestRecord>"
            ),
            Ok(Type::HashMap(
                Box::new(Type::UInt32),
                Box::new(Type::Record {
                    module_path: "types".into(),
                    name: "TestRecord".into(),
                })
            ))
        );
        // Resolution edge case, the generic type is in a different module than its arguments
        assert_eq!(
            run_resolve_type(
                &ir,
                &mut cache,
                "types::mod1",
                "super::HashMap<u32, Mod1Record>"
            ),
            Ok(Type::HashMap(
                Box::new(Type::UInt32),
                Box::new(Type::Record {
                    module_path: "types::mod1".into(),
                    name: "Mod1Record".into(),
                })
            ))
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "Option::<mod1::Mod1Record>"),
            Ok(Type::Option(Box::new(Type::Record {
                module_path: "types::mod1".into(),
                name: "Mod1Record".into(),
            })))
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "Arc<u8>"),
            Ok(Type::Arc(Box::new(Type::UInt8))),
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "Box<String>"),
            Ok(Type::Box(Box::new(Type::String))),
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "Arc<Arc<Box<String>>>"),
            Ok(Type::Arc(Box::new(Type::Arc(Box::new(Type::Box(
                Box::new(Type::String)
            )))))),
        );

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "Result<u8, TestError>"),
            Ok(Type::Result(
                Box::new(Type::UInt8),
                Box::new(Type::Enum {
                    module_path: "types".into(),
                    name: "TestError".into(),
                }),
            ))
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "Result<(), TestError>"),
            Ok(Type::Result(
                Box::new(Type::Unit),
                Box::new(Type::Enum {
                    module_path: "types".into(),
                    name: "TestError".into(),
                }),
            ))
        );
        assert_eq!(
            run_resolve_type(
                &ir,
                &mut cache,
                "types",
                "::std::result::Result<(), TestError>"
            ),
            Ok(Type::Result(
                Box::new(Type::Unit),
                Box::new(Type::Enum {
                    module_path: "types".into(),
                    name: "TestError".into(),
                }),
            ))
        );
    }

    #[test]
    fn test_resolve_type_trait_objects() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "dyn TraitInterface"),
            Ok(Type::Trait {
                module_path: "types::mod1".into(),
                name: "TraitInterface".into(),
                export_ty: TraitExportType::TraitInterface,
            })
        );
        assert_eq!(
            run_resolve_type(
                &ir,
                &mut cache,
                "types::mod1",
                "dyn TraitInterfaceWithForeign"
            ),
            Ok(Type::Trait {
                module_path: "types::mod1".into(),
                name: "TraitInterfaceWithForeign".into(),
                export_ty: TraitExportType::TraitInterfaceWithForeign,
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "dyn CallbackInterface"),
            Ok(Type::Trait {
                module_path: "types::mod1".into(),
                name: "CallbackInterface".into(),
                export_ty: TraitExportType::CallbackInterface,
            })
        );

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "TraitInterface"),
            Err(ErrorKind::TraitWithoutDyn)
        );

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "dyn MyTrait + Send"),
            Err(ErrorKind::InvalidDynTrait)
        );

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "dyn MyRecord"),
            Err(ErrorKind::NotFound)
        );
    }

    #[test]
    fn test_resolve_type_with_type_aliases() {
        let ir = Ir::new_for_test(&["type_aliases"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "RecordAlias"),
            Ok(Type::Record {
                module_path: "type_aliases".into(),
                name: "Record".into(),
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "SubmoduleRecordAlias"),
            Ok(Type::Record {
                module_path: "type_aliases::submod".into(),
                name: "Record".into(),
            })
        );
        assert_eq!(
            run_resolve_type(
                &ir,
                &mut cache,
                "type_aliases::submod2",
                "SubmoduleRecordAlias"
            ),
            Ok(Type::Record {
                module_path: "type_aliases::submod".into(),
                name: "Record".into(),
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "UnitAlias"),
            Ok(Type::Unit)
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "FileResult<Record>"),
            Ok(Type::Result(
                Box::new(Type::Record {
                    module_path: "type_aliases".into(),
                    name: "Record".into(),
                }),
                Box::new(Type::Enum {
                    module_path: "type_aliases".into(),
                    name: "FileError".into(),
                })
            ))
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "FileResult2<()>"),
            Ok(Type::Result(
                Box::new(Type::Unit),
                Box::new(Type::Enum {
                    module_path: "type_aliases".into(),
                    name: "FileError".into(),
                })
            ))
        );
        assert_eq!(
            run_resolve_type(
                &ir,
                &mut cache,
                "type_aliases",
                "FileResult2<Record, OtherError>"
            ),
            Ok(Type::Result(
                Box::new(Type::Record {
                    module_path: "type_aliases".into(),
                    name: "Record".into(),
                }),
                Box::new(Type::Enum {
                    module_path: "type_aliases".into(),
                    name: "OtherError".into(),
                }),
            ))
        );
        assert_eq!(
            run_resolve_type(
                &ir,
                &mut cache,
                "type_aliases::submod",
                "super::FileResult2<Record>"
            ),
            Ok(Type::Result(
                Box::new(Type::Record {
                    module_path: "type_aliases::submod".into(),
                    name: "Record".into(),
                }),
                Box::new(Type::Enum {
                    module_path: "type_aliases".into(),
                    name: "FileError".into(),
                }),
            ))
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "&Self"),
            Ok(Type::Ref {
                mutable: false,
                ty: Box::new(Type::SelfTy)
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "&mut Self"),
            Ok(Type::Ref {
                mutable: true,
                ty: Box::new(Type::SelfTy)
            })
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "Arc<Self>"),
            Ok(Type::Arc(Box::new(Type::SelfTy)))
        );
        // This should fail because `RecordAlias` is in not defined in `type_aliases::submod`, even
        // though it is defined in the module that `FileResult2` lives in
        assert_eq!(
            run_resolve_type(
                &ir,
                &mut cache,
                "type_aliases::submod",
                "super::FileResult2<RecordAlias>"
            ),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "FileResult3"),
            Ok(Type::Result(
                Box::new(Type::UInt32),
                Box::new(Type::Enum {
                    module_path: "type_aliases".into(),
                    name: "FileError".into(),
                }),
            ))
        );

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "CircularAlias"),
            Err(ErrorKind::CycleDetected),
        );
        assert_eq!(
            run_resolve_type(&ir, &mut cache, "type_aliases", "FileResult<FileResult>"),
            Err(ErrorKind::MissingGenericArg),
        );
    }

    #[test]
    fn test_resolve_box_types() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        // Right now we just ignore the box since it doesn't make a difference in the generated
        // bindings.  If we want to use this to generate scaffolding, then we'll need something
        // like a uniffi_meta::Box variant.

        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "types::mod1", "Box<u32>", None),
            Ok(uniffi_meta::Type::Box {
                inner_type: Box::new(uniffi_meta::Type::UInt32),
            })
        );

        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "types", "Box<TestRecord>", None),
            Ok(uniffi_meta::Type::Box {
                inner_type: Box::new(uniffi_meta::Type::Record {
                    module_path: "types".into(),
                    name: "TestRecord".into(),
                })
            })
        );
    }

    #[test]
    fn test_resolve_type_references() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types::mod1", "&std::primitive::u32"),
            Ok(Type::Ref {
                mutable: false,
                ty: Box::new(Type::UInt32),
            })
        );

        assert_eq!(
            run_resolve_type(&ir, &mut cache, "types", "&Result<TestRecord, TestError>"),
            Ok(Type::Ref {
                mutable: false,
                ty: Box::new(Type::Result(
                    Box::new(Type::Record {
                        module_path: "types".into(),
                        name: "TestRecord".into(),
                    }),
                    Box::new(Type::Enum {
                        module_path: "types".into(),
                        name: "TestError".into(),
                    }),
                ))
            })
        );
    }

    #[test]
    fn test_remote_types() {
        let ir = Ir::new_for_test(&["remote_types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "remote_types", "AnyhowError", None),
            Ok(uniffi_meta::Type::Object {
                module_path: "remote_types".into(),
                name: "AnyhowError".into(),
                imp: ObjectImpl::Struct,
            })
        );
        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "remote_types", "LogLevel", None),
            Ok(uniffi_meta::Type::Enum {
                module_path: "remote_types".into(),
                name: "LogLevel".into(),
            })
        );
    }

    #[test]
    fn test_result_uniffi_meta() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "types", "TestRecord", None),
            Ok(uniffi_meta::Type::Record {
                module_path: "types".into(),
                name: "TestRecord".into(),
            }),
        );

        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "types", "Arc<TestInterface>", None),
            Ok(uniffi_meta::Type::Object {
                module_path: "types".into(),
                name: "TestInterface".into(),
                imp: ObjectImpl::Struct,
            }),
        );

        assert_eq!(
            run_resolve_uniffi_meta_type(
                &ir,
                &mut cache,
                "types",
                "Arc<dyn mod1::TraitInterface>",
                None
            ),
            Ok(uniffi_meta::Type::Object {
                module_path: "types".into(),
                name: "TraitInterface".into(),
                imp: ObjectImpl::Trait,
            }),
        );

        let self_ty = uniffi_meta::Type::Object {
            module_path: "types".into(),
            name: "TraitInterface".into(),
            imp: ObjectImpl::Trait,
        };
        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "types", "Arc<Self>", Some(&self_ty)),
            Ok(uniffi_meta::Type::Object {
                module_path: "types".into(),
                name: "TraitInterface".into(),
                imp: ObjectImpl::Trait,
            }),
        );
    }

    #[test]
    fn test_result_arg() {
        let ir = Ir::new_for_test(&["types"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_arg(&ir, &mut cache, "types", "TestRecord", None),
            Ok(ArgType {
                ty: uniffi_meta::Type::Record {
                    module_path: "types".into(),
                    name: "TestRecord".into(),
                },
                by_ref: false,
            }),
        );
        assert_eq!(
            run_resolve_arg(&ir, &mut cache, "types", "&TestRecord", None),
            Ok(ArgType {
                ty: uniffi_meta::Type::Record {
                    module_path: "types".into(),
                    name: "TestRecord".into(),
                },
                by_ref: true,
            }),
        );
        assert_eq!(
            run_resolve_arg(&ir, &mut cache, "types", "&dyn mod1::TraitInterface", None),
            Ok(ArgType {
                ty: uniffi_meta::Type::Object {
                    module_path: "types".into(),
                    name: "TraitInterface".into(),
                    imp: ObjectImpl::Trait,
                },
                by_ref: true,
            }),
        );
        assert_eq!(
            run_resolve_arg(&ir, &mut cache, "types", "&[TestRecord]", None),
            Ok(ArgType {
                ty: uniffi_meta::Type::Sequence {
                    inner_type: Box::new(uniffi_meta::Type::Record {
                        module_path: "types".into(),
                        name: "TestRecord".into(),
                    }),
                },
                by_ref: true,
            }),
        );
    }

    #[test]
    fn test_raw_ident() {
        let ir = Ir::new_for_test(&["raw_idents"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "raw_idents", "RecordWrapper", None),
            Ok(uniffi_meta::Type::Custom {
                module_path: "raw_idents".into(),
                name: "RecordWrapper".into(),
                builtin: Box::new(uniffi_meta::Type::Record {
                    module_path: "raw_idents".into(),
                    name: "Record".into(),
                })
            })
        );
        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "raw_idents", "RecordWrapper2", None),
            Ok(uniffi_meta::Type::Custom {
                module_path: "raw_idents".into(),
                name: "RecordWrapper2".into(),
                builtin: Box::new(uniffi_meta::Type::Record {
                    module_path: "raw_idents".into(),
                    name: "Record".into(),
                })
            })
        );
        assert_eq!(
            run_resolve_uniffi_meta_type(&ir, &mut cache, "raw_idents", "Guid", None),
            Ok(uniffi_meta::Type::Custom {
                module_path: "raw_idents".into(),
                name: "Guid".into(),
                builtin: Box::new(uniffi_meta::Type::UInt64),
            })
        );
    }
}
