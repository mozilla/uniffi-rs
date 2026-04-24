/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Lookup items for paths
//!
//! Also handles resolving types, which is closely related.

use core::fmt;
use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use syn::{ext::IdentExt, spanned::Spanned, Ident, Path};

use crate::{
    files::FileId, BuiltinItem, Error, ErrorKind::*, Ir, Item, ItemNames, Module, Result, UseGlob,
    UseItem,
};

// For tests only, print tracing info.
// This makes it easier to debug errors in path resolution
macro_rules! trace {
    ($($tt:tt)*) => {
        #[cfg(test)]
        println!($($tt)*);
    }
}

// Path where all Idents have been resolved to Items
#[derive(Debug, Clone)]
pub struct RPath<'ir> {
    items: Vec<&'ir Item>,
}

impl<'ir> RPath<'ir> {
    pub fn new(crate_root: &'ir Item) -> Self {
        Self {
            items: vec![crate_root],
        }
    }

    /// Get the item this path points to
    pub fn item(&self) -> Result<&'ir Item> {
        match self.items.last() {
            Some(i) => Ok(i),
            None => Err(Error::internal("Path::item(): items is empty")),
        }
    }

    pub fn push(&mut self, item: &'ir Item) {
        self.items.push(item);
    }

    pub fn pop(&mut self) {
        self.items.pop();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn parent(&self) -> Result<Self> {
        match self.items.as_slice() {
            [] => Err(Error::internal("Path::parent_module: items is empty")),
            [_] => Err(Error::internal(
                "Path::parent_module: items only has the crate",
            )),
            [start @ .., _] => Ok(Self {
                items: Vec::from(start),
            }),
        }
    }

    pub fn crate_root(&self) -> Result<Self> {
        match self.items.as_slice() {
            [] => Err(Error::internal("Path::crate_root: items is empty")),
            [first, ..] => Ok(Self { items: vec![first] }),
        }
    }

    pub fn crate_root_module(&self) -> Result<&'ir Module> {
        self.crate_root()?.module()
    }

    pub fn append_child(&self, item: &'ir Item) -> Self {
        Self {
            items: Vec::from_iter(self.items.iter().cloned().chain([item])),
        }
    }

    pub fn module(&self) -> Result<&'ir Module> {
        self.items
            .iter()
            .rev()
            .find_map(|i| match i {
                Item::Module(m) => Some(m),
                _ => None,
            })
            .ok_or(Error::internal("Path::module: no modules found"))
    }

    pub fn file_id(&self) -> FileId {
        self.module().expect("file_id failed").source
    }

    pub fn path_string(&self) -> String {
        let mut path = String::new();
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                path.push_str("::");
            }
            if let Item::Builtin(builtin) = item {
                match builtin {
                    BuiltinItem::UnitType => path.push_str("()"),
                    BuiltinItem::Boolean => path.push_str("bool"),
                    BuiltinItem::String => path.push_str("String"),
                    BuiltinItem::Str => path.push_str("str"),
                    BuiltinItem::UInt8 => path.push_str("u8"),
                    BuiltinItem::Int8 => path.push_str("i8"),
                    BuiltinItem::UInt16 => path.push_str("u16"),
                    BuiltinItem::Int16 => path.push_str("i16"),
                    BuiltinItem::UInt32 => path.push_str("u32"),
                    BuiltinItem::Int32 => path.push_str("i32"),
                    BuiltinItem::UInt64 => path.push_str("u64"),
                    BuiltinItem::Int64 => path.push_str("i64"),
                    BuiltinItem::Float32 => path.push_str("f32"),
                    BuiltinItem::Float64 => path.push_str("f64"),
                    BuiltinItem::SystemTime => path.push_str("Timestamp"),
                    BuiltinItem::Duration => path.push_str("Duration"),
                    BuiltinItem::Vec => path.push_str("Vec"),
                    BuiltinItem::Arc => path.push_str("Arc"),
                    BuiltinItem::Box => path.push_str("Box"),
                    BuiltinItem::HashMap => path.push_str("HashMap"),
                    BuiltinItem::Option => path.push_str("Option"),
                    BuiltinItem::Result => path.push_str("Result"),
                    BuiltinItem::UniffiMacro(name) => path.push_str(name),
                }
            } else {
                path.push_str(&item.name());
            }
        }
        path
    }

    /// Resolve a syn Path into an RPath
    ///
    /// This is an instance method, since paths are resolved relative to an existing path.
    pub fn resolve(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &Path,
        namespace: Namespace,
    ) -> Result<Self> {
        trace!(
            "Path::resolve {} -- {}",
            self.path_string(),
            quote::quote! { #path }
        );

        if path.leading_colon.is_some() || self.items.is_empty() {
            return self.resolve_global_path(ir, cache, path, namespace);
        }

        let mut current_path = self.clone();
        if path.segments.is_empty() {
            return Err(Error::new(self.file_id(), path.span(), NotFound));
        }
        for (i, seg) in path.segments.iter().enumerate() {
            // Use the type namespace for all items except the last,
            // since we always want to match modules
            let child_namespace = if i < path.segments.len() - 1 {
                Namespace::Type
            } else {
                namespace
            };
            trace!(
                "  resolve (path: {}, ident: {} namespace: {child_namespace:?})",
                current_path.path_string(),
                seg.ident
            );
            current_path = match current_path.child(ir, cache, &seg.ident, child_namespace) {
                Ok(child_item) => child_item.path,
                // For the first segment only, try falling back to a global lookup on lookup errors
                Err(e) if i == 0 && e.is_not_found() => {
                    trace!("  PathError::NotFound, try global lookup");
                    return self.resolve_global_path(ir, cache, path, namespace);
                }
                Err(e) => return Err(e),
            }
        }
        trace!("  resolved: {}", current_path.path_string());
        Ok(current_path)
    }

    pub fn resolve_global_path(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &Path,
        namespace: Namespace,
    ) -> Result<Self> {
        trace!("Path::resolve_global_path {}", quote::quote! { #path });
        if path.segments.is_empty() {
            return Err(Error::new(self.file_id(), path.span(), NotFound));
        }

        let first_ident = &path.segments.first().unwrap().ident;

        // Lookup UDL item from the crate root
        if path.segments.len() == 1 {
            if let Some(item) = self.crate_root_module()?.lookup_udl_item(first_ident) {
                trace!("  resolved to UDL item: {item:?}");
                let mut path = RPath::new(self.items[0]);
                path.push(item);
                return Ok(path);
            }
        }

        match ir.crate_roots.get(first_ident) {
            Some(crate_root) => {
                let mut rpath = RPath::new(crate_root);
                trace!("  found crate root (path: {})", rpath.path_string());
                for (i, seg) in path.segments.iter().enumerate().skip(1) {
                    trace!(
                        "  resolve_global_path (path: {}, ident: {})",
                        rpath.path_string(),
                        seg.ident
                    );
                    // Use the type namespace for all items except the last,
                    // since we always want to match modules
                    let child_namespace = if i < path.segments.len() - 1 {
                        Namespace::Type
                    } else {
                        namespace
                    };
                    rpath = rpath.child(ir, cache, &seg.ident, child_namespace)?.path;
                }
                trace!("  resolved: {}", rpath.path_string());
                Ok(rpath)
            }
            None => match get_builtin_item(path) {
                Some(item) => {
                    trace!("  resolved to builtin: {item:?}");
                    Ok(RPath::new(item))
                }
                None => {
                    trace!("  not found");
                    Err(Error::new(self.file_id(), path.span(), NotFound))
                }
            },
        }
    }

    /// Get a child item for this path
    ///
    /// The child item path will be the canonical path for the item.
    /// When looking up `SomeItem` from a module with a `use crate::foo::bar::SomeItem` statement,
    /// the returned path will be `foo::bar::SomeItem` instead of `[current_path]::SomeItem`
    pub fn child(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        ident: &Ident,
        namespace: Namespace,
    ) -> Result<ChildItem<'ir>> {
        // Special case `super` and `crate`, no need to check the cache for this one.
        if ident == "super" {
            return match self.parent() {
                Ok(path) => Ok(ChildItem {
                    path,
                    vis: Visibility::Public,
                }),
                Err(_) => Err(Error::new(self.file_id(), ident.span(), SuperInvalid)),
            };
        } else if ident == "crate" {
            return match self.crate_root() {
                Ok(path) => Ok(ChildItem {
                    path,
                    vis: Visibility::Public,
                }),
                Err(_) => Err(Error::new(self.file_id(), ident.span(), CrateInvalid)),
            };
        }

        let Item::Module(module) = self.item()? else {
            // We currently assume non-module items have no child items
            return Err(Error::new(self.file_id(), ident.span(), NotFound));
        };
        let key = (module.id, ident.clone(), namespace);
        if cache.children_resolving.contains(&key) {
            return Err(Error::new(self.file_id(), ident.span(), CycleDetected));
        }

        if let Some(result) = cache.children.get(&key) {
            return result.clone();
        }

        cache.children_resolving.insert(key.clone());
        let result = self._child(ir, cache, module, ident, namespace);
        cache.children_resolving.remove(&key);
        cache.children.insert(key, result.clone());
        result
    }

    /// Non-caching part of `child`.
    ///
    /// This implements the logic of `child`, but doesn't handle the cache for this ident/item
    /// pair.  However, it still inputs the cache and uses it when resolving items from a `use`
    /// statement.
    fn _child(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module: &'ir Module,
        ident: &Ident,
        namespace: Namespace,
    ) -> Result<ChildItem<'ir>> {
        let ident = ident.unraw();
        if let Some(path) = self.child_udl_item(module, &ident, namespace)? {
            return Ok(ChildItem {
                path,
                vis: Visibility::Public,
            });
        }
        if let Some(child) = self.child_special_item(ir, cache, module, &ident, namespace)? {
            return Ok(child);
        }
        let mut use_globs = vec![];
        if let Some(child) =
            self.child_item_or_non_glob_use(ir, cache, module, &ident, &mut use_globs, namespace)?
        {
            return Ok(child);
        }
        if let Some(child) = self.child_glob_use(ir, cache, &ident, use_globs, namespace)? {
            Ok(child)
        } else {
            Err(Error::new(self.file_id(), ident.span(), NotFound))
        }
    }

    /// Try to find a UDL item for [Self::child]
    fn child_udl_item(
        &self,
        module: &'ir Module,
        ident: &Ident,
        namespace: Namespace,
    ) -> Result<Option<Self>> {
        match module.lookup_udl_item(ident) {
            Some(item) if namespace.matches(item) => Ok(Some(self.append_child(item))),
            _ => Ok(None),
        }
    }

    // Try to find "special" items like `uniffi::use_remote_type!` for [Self::child]
    //
    // These don't represent real Rust items, they're more like instructions to UniFFI.
    fn child_special_item(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module: &'ir Module,
        ident: &Ident,
        namespace: Namespace,
    ) -> Result<Option<ChildItem<'ir>>> {
        let mut found = None;
        for item in module
            .items
            .iter()
            .filter(|i| i.is_special() && namespace.matches(i))
        {
            if let Some(item_ident) = item.ident() {
                if item_ident != *ident {
                    continue;
                }
                match found {
                    None => found = Some((item_ident, item)),
                    Some((prev_ident, _)) => {
                        return Err(Error::new(self.file_id(), prev_ident.span(), NameConflict)
                            .context(self.file_id(), prev_ident.span(), "previous item"))
                    }
                }
            }
        }
        match found {
            Some((_, Item::UseRemoteType(path))) => Ok(Some(ChildItem {
                path: self.resolve(ir, cache, path, namespace)?,
                vis: Visibility::Public,
            })),
            Some((_, item)) => Ok(Some(ChildItem {
                path: self.append_child(item),
                vis: Visibility::Public,
            })),
            None => Ok(None),
        }
    }

    // Try to find a module child or an item from a non-glob use statement for [Self::child]
    //
    // While we're looking for these items, we also push any use glob's we see to `use_globs`
    fn child_item_or_non_glob_use(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module: &'ir Module,
        ident: &Ident,
        use_globs: &mut Vec<&'ir UseGlob>,
        namespace: Namespace,
    ) -> Result<Option<ChildItem<'ir>>> {
        enum FoundItem<'ir> {
            Use(&'ir UseItem, RPath<'ir>),
            Item(Ident, &'ir Item),
        }
        impl FoundItem<'_> {
            fn span(&self) -> Span {
                match self {
                    Self::Use(item_use, _) => item_use.ident.span(),
                    Self::Item(ident, _) => ident.span(),
                }
            }
            fn matches_use(&self, path: RPath<'_>) -> bool {
                match self {
                    Self::Use(_, p) => *p == path,
                    Self::Item(_, _) => false,
                }
            }
        }
        let mut found: Option<FoundItem<'ir>> = None;
        for item in module.items.iter() {
            if let Some(item_ident) = item.ident() {
                if &item_ident == ident && namespace.matches(item) {
                    if let Some(found) = found {
                        return Err(Error::new(self.file_id(), item_ident.span(), NameConflict)
                            .context(self.file_id(), found.span(), "previous item"));
                    }
                    found = Some(FoundItem::Item(item_ident, item));
                }
            } else {
                match item {
                    Item::UseItem(use_item) if use_item.ident == *ident => {
                        let resolved = match self.resolve(ir, cache, &use_item.path, namespace) {
                            Ok(p) => p,
                            // ignore not found errors, maybe this is an item from another
                            // crate.
                            Err(e) if e.is_not_found() => continue,
                            Err(e) => {
                                return Err(e.context(
                                    self.file_id(),
                                    use_item.span,
                                    "while resolving use",
                                ))
                            }
                        };
                        if namespace.matches(resolved.item()?) {
                            if let Some(found) = &found {
                                if found.matches_use(resolved) {
                                    // If multiple use statements resolve to the same item, that's
                                    // okay just skip the extra ones.
                                    continue;
                                }
                                return Err(Error::new(
                                    self.file_id(),
                                    use_item.span,
                                    NameConflict,
                                )
                                .context(
                                    self.file_id(),
                                    found.span(),
                                    "previous item",
                                ));
                            }
                            found = Some(FoundItem::Use(use_item, resolved));
                        }
                    }
                    Item::UseGlob(use_glob) => {
                        // Not used now, but let's store for the next step
                        use_globs.push(use_glob);
                    }
                    _ => (),
                }
            }
        }
        match found {
            Some(FoundItem::Item(_, item)) => Ok(Some(ChildItem {
                path: self.append_child(item),
                vis: item.vis(),
            })),
            Some(FoundItem::Use(use_item, path)) => Ok(Some(ChildItem {
                path: path.clone(),
                vis: use_item.vis,
            })),
            None => Ok(None),
        }
    }

    // Try to find an item from a glob use statement for [Self::child]
    fn child_glob_use(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        ident: &Ident,
        use_globs: Vec<&'ir UseGlob>,
        namespace: Namespace,
    ) -> Result<Option<ChildItem<'ir>>> {
        let mut found = None;

        for use_glob in use_globs {
            let mut path = use_glob.module_path.clone();
            path.segments.push(ident.clone().into());

            let path = match self.resolve(ir, cache, &path, namespace) {
                Ok(path) => path,
                Err(e) if e.is_not_found() || e.is_cycle_detected() => continue,
                Err(e) => {
                    return Err(e.context(
                        self.file_id(),
                        use_glob.star_token.span(),
                        "while resolving glob",
                    ))
                }
            };
            match &found {
                None => found = Some((path, use_glob)),
                Some((prev_path, _)) => {
                    if *prev_path != path {
                        return Err(Error::new(
                            self.file_id(),
                            use_glob.star_token.span(),
                            NameConflict,
                        )
                        .context(
                            self.file_id(),
                            use_glob.star_token.span(),
                            "previous item",
                        ));
                    }
                }
            }
        }

        match found {
            None => Ok(None),
            Some((path, use_glob)) => Ok(Some(ChildItem {
                vis: use_glob.vis,
                path,
            })),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChildItem<'ir> {
    pub vis: Visibility,
    pub path: RPath<'ir>,
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Public,
    Private,
}

impl Visibility {
    pub fn is_pub(&self) -> bool {
        matches!(self, Self::Public)
    }
}

impl From<syn::Visibility> for Visibility {
    fn from(vis: syn::Visibility) -> Self {
        match vis {
            syn::Visibility::Public(_) => Self::Public,
            _ => Self::Private,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    Value,
    Type,
    Macro,
    // Special-cased namespace that only matches Item::NonUniffi types.
    // This is used to find the concrete type for a custom type.
    NonUniffiType,
}

impl Namespace {
    pub fn matches(&self, item: &Item) -> bool {
        match self {
            Self::Type => matches!(
                item,
                Item::Module(_)
                    | Item::Record(_)
                    | Item::Enum(_)
                    | Item::Object(_)
                    | Item::Trait(_)
                    | Item::CustomType(_)
                    | Item::Udl(_)
                    | Item::Type(_)
                    | Item::Builtin(_)
                    | Item::UseRemoteType(_)
            ),
            Self::Value => matches!(item, Item::Fn(_),),
            Self::Macro => matches!(item, Item::Builtin(BuiltinItem::UniffiMacro(_)),),
            Self::NonUniffiType => matches!(item, Item::NonUniffi(_, _),),
        }
    }
}

impl PartialEq for RPath<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Paths are equal if the references point to the same object.
        // We only need to check the final item to know this.
        match (self.items.last(), other.items.last()) {
            (None, None) => true,
            (Some(a), Some(b)) => std::ptr::eq::<Item>(*a, *b),
            _ => false,
        }
    }
}

impl Eq for RPath<'_> {}

impl fmt::Display for RPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path_string())
    }
}

/// Cache for path lookups
#[derive(Default)]
pub struct LookupCache<'ir> {
    // Cached results for `RPath::child`
    // This maps module id / ident / namespaces to lookup results
    pub children: HashMap<(usize, Ident, Namespace), Result<ChildItem<'ir>>>,
    // Module children we're currently resolving, this is used to detect cycles
    pub children_resolving: HashSet<(usize, Ident, Namespace)>,
    // Cached results for `RPath::public_path_to_item()`
    // This maps path strings to lookup results
    pub public_paths: HashMap<String, Result<ItemNames>>,
}

fn get_builtin_item(path: &Path) -> Option<&'static Item> {
    let path = path
        .segments
        .iter()
        .map(|seg| seg.ident.unraw().to_string())
        .collect::<Vec<_>>()
        .join("::");

    match path.as_str() {
        "std::primitive::unit" | "core::primitive::unit" => {
            Some(&Item::Builtin(BuiltinItem::UnitType))
        }
        "bool" | "std::primitive::bool" | "core::primitive::bool" => {
            Some(&Item::Builtin(BuiltinItem::Boolean))
        }
        "u8" | "std::primitive::u8" | "core::primitive::u8" => {
            Some(&Item::Builtin(BuiltinItem::UInt8))
        }
        "i8" | "std::primitive::i8" | "core::primitive::i8" => {
            Some(&Item::Builtin(BuiltinItem::Int8))
        }
        "u16" | "std::primitive::u16" | "core::primitive::u16" => {
            Some(&Item::Builtin(BuiltinItem::UInt16))
        }
        "i16" | "std::primitive::i16" | "core::primitive::i16" => {
            Some(&Item::Builtin(BuiltinItem::Int16))
        }
        "u32" | "std::primitive::u32" | "core::primitive::u32" => {
            Some(&Item::Builtin(BuiltinItem::UInt32))
        }
        "i32" | "std::primitive::i32" | "core::primitive::i32" => {
            Some(&Item::Builtin(BuiltinItem::Int32))
        }
        "u64" | "std::primitive::u64" | "core::primitive::u64" => {
            Some(&Item::Builtin(BuiltinItem::UInt64))
        }
        "i64" | "std::primitive::i64" | "core::primitive::i64" => {
            Some(&Item::Builtin(BuiltinItem::Int64))
        }
        "f32" | "std::primitive::f32" | "core::primitive::f32" => {
            Some(&Item::Builtin(BuiltinItem::Float32))
        }
        "f64" | "std::primitive::f64" | "core::primitive::f64" => {
            Some(&Item::Builtin(BuiltinItem::Float64))
        }
        "Option" | "std::option::Option" => Some(&Item::Builtin(BuiltinItem::Option)),
        "Box" | "std::boxed::Box" => Some(&Item::Builtin(BuiltinItem::Box)),
        "Vec" | "std::vec::Vec" => Some(&Item::Builtin(BuiltinItem::Vec)),
        "HashMap" | "std::collections::HashMap" => Some(&Item::Builtin(BuiltinItem::HashMap)),
        "Arc" | "std::sync::Arc" => Some(&Item::Builtin(BuiltinItem::Arc)),
        "Result" | "std::result::Result" => Some(&Item::Builtin(BuiltinItem::Result)),
        "std::time::SystemTime" => Some(&Item::Builtin(BuiltinItem::SystemTime)),
        "std::time::Duration" => Some(&Item::Builtin(BuiltinItem::Duration)),
        "String" | "std::string::String" => Some(&Item::Builtin(BuiltinItem::String)),
        "str" | "std::primitive::str" => Some(&Item::Builtin(BuiltinItem::Str)),
        "uniffi::custom_type" => Some(&Item::Builtin(BuiltinItem::UniffiMacro("custom_type"))),
        "uniffi::custom_newtype" => {
            Some(&Item::Builtin(BuiltinItem::UniffiMacro("custom_newtype")))
        }
        "uniffi::use_remote_type" => {
            Some(&Item::Builtin(BuiltinItem::UniffiMacro("use_remote_type")))
        }
        _ => None,
    }
}

#[cfg(test)]
pub mod tests {
    use quote::format_ident;

    use crate::ErrorKind;

    use super::*;

    fn run_resolve_item<'ir>(
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &str,
        path: &str,
    ) -> Result<String, ErrorKind> {
        let rpath = path_for_module(ir, module_path);
        rpath
            .resolve(ir, cache, &syn::parse_str(path).unwrap(), Namespace::Type)
            .map(|path| format!("{path}"))
            .map_err(|e| e.kind)
    }

    fn run_resolve_item_value_namespace<'ir>(
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &str,
        path: &str,
    ) -> Result<String, ErrorKind> {
        let rpath = path_for_module(ir, module_path);
        rpath
            .resolve(ir, cache, &syn::parse_str(path).unwrap(), Namespace::Value)
            .map(|path| format!("{path}"))
            .map_err(|e| {
                println!("{e}");
                e.kind
            })
    }

    pub fn path_for_module<'ir>(ir: &'ir Ir, module_path: &str) -> RPath<'ir> {
        let mut parts = module_path.split("::");
        let crate_name = parts.next().unwrap();
        let item = ir
            .crate_roots
            .get(&format_ident!("{crate_name}"))
            .unwrap_or_else(|| panic!("crate root not found: {crate_name}"));
        let mut module = match item {
            Item::Module(module) => module,
            _ => panic!("Crate root not module"),
        };
        let mut path = RPath::new(item);
        for module_name in parts {
            let child_item = module
                .items
                .iter()
                .find(|item| matches!(item, Item::Module(child) if child.ident == module_name))
                .unwrap_or_else(|| panic!("module not found ({module_name}) ({module_path})"));

            if let Item::Module(child_mod) = child_item {
                module = child_mod;
                path.push(child_item);
            } else {
                unreachable!()
            }
        }
        path
    }

    #[test]
    fn test_resolve_item() {
        let ir = Ir::new_for_test(&["paths"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "Mod1Record"),
            Ok("paths::mod1::Mod1Record".to_string()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1::mod2", "mod3::Mod3Record"),
            Ok("paths::mod1::mod2::mod3::Mod3Record".to_string()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "missing_mod::MyRecord"),
            Err(ErrorKind::NotFound),
        );
    }

    #[test]
    fn test_resolve_item_with_super_keyword() {
        let ir = Ir::new_for_test(&["paths"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(
                &ir,
                &mut cache,
                "paths::mod1::mod2::mod3",
                "super::Mod2Record"
            ),
            Ok("paths::mod1::mod2::Mod2Record".into()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "super::mod4::Mod4Record"),
            Ok("paths::mod4::Mod4Record".into()),
        );
        assert_eq!(
            run_resolve_item(
                &ir,
                &mut cache,
                "paths::mod1",
                "super::missing_mod::MyRecord"
            ),
            Err(ErrorKind::NotFound),
        );

        // Super should fail when used in the top-level module
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths", "super::MyRecord"),
            Err(ErrorKind::SuperInvalid),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "super::super::MyRecord"),
            Err(ErrorKind::SuperInvalid),
        );
    }

    #[test]
    fn test_resolve_item_with_self_keyword() {
        let ir = Ir::new_for_test(&["paths"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths", "mod3::Mod3Record"),
            Ok("paths::mod1::mod2::mod3::Mod3Record".to_string()),
        );
    }

    #[test]
    fn test_resolve_item_with_crate_keyword() {
        let ir = Ir::new_for_test(&["paths"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(
                &ir,
                &mut cache,
                "paths::mod1::mod2::mod3",
                "crate::TestRecord"
            ),
            Ok("paths::TestRecord".into())
        );
        assert_eq!(
            run_resolve_item(
                &ir,
                &mut cache,
                "paths::mod1::mod2::mod3",
                "crate::mod4::Mod4Record"
            ),
            Ok("paths::mod4::Mod4Record".into())
        );
    }

    #[test]
    fn test_resolve_item_with_rust_keyword() {
        let ir = Ir::new_for_test(&["paths"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths", "r#break"),
            Ok("paths::break".into())
        );

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "super::r#break"),
            Ok("paths::break".into())
        );
    }

    #[test]
    fn test_use_remote_type() {
        let ir = Ir::new_for_test(&["paths", "paths2", "paths3"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths", "RemoteRecord"),
            Ok("paths3::RemoteRecord".into())
        );

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths", "Url"),
            Ok("paths3::Url".into())
        );
    }

    #[test]
    fn test_resolve_item_with_implicate_crate_lookup() {
        let ir = Ir::new_for_test(&["paths", "paths2"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            // `paths2` doesn't exist in `mod3`, so this should lookup a top-level crate
            run_resolve_item(
                &ir,
                &mut cache,
                "paths::mod1::mod2::mod3",
                "paths2::AmbiguousRecord"
            ),
            Ok("paths2::AmbiguousRecord".into())
        );
        assert_eq!(
            // `paths2` exists in the `paths` root module, so we should use that
            run_resolve_item(&ir, &mut cache, "paths", "paths2::AmbiguousRecord"),
            Ok("paths::paths2::AmbiguousRecord".into())
        );
        assert_eq!(
            // If there's a leading `::` then we should always do a crate lookup
            run_resolve_item(&ir, &mut cache, "paths", "::paths2::AmbiguousRecord"),
            Ok("paths2::AmbiguousRecord".into())
        );
    }

    #[test]
    fn test_resolve_item_with_use() {
        let ir = Ir::new_for_test(&["paths", "paths2"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths2", "TestRecord"),
            Ok("paths::TestRecord".into()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths2", "mod2::mod3::Mod3Record"),
            Ok("paths::mod1::mod2::mod3::Mod3Record".into()),
        );

        // renamed import
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "Mod2RecordRenamed",),
            Ok("paths::mod1::mod2::Mod2Record".into()),
        );

        // glob import
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "Mod3Record",),
            Ok("paths::mod1::mod2::mod3::Mod3Record".into()),
        );

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "CircularUseImport"),
            Err(ErrorKind::CycleDetected),
        );
    }

    #[test]
    fn test_name_conflicts() {
        let ir = Ir::new_for_test(&["name_conflicts"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "name_conflicts", "Record"),
            Ok("name_conflicts::Record".into()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "name_conflicts", "RenamedRecordConflict"),
            Err(ErrorKind::NameConflict),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "name_conflicts", "ItemGlobConflict"),
            Ok("name_conflicts::ItemGlobConflict".into()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "name_conflicts", "GlobGlobConflict"),
            Err(ErrorKind::NameConflict),
        );

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "name_conflicts", "CustomTypeConflict"),
            Err(ErrorKind::NameConflict),
        );

        // Check resolving paths to functions, which use the value namespace.
        assert_eq!(
            run_resolve_item_value_namespace(
                &ir,
                &mut cache,
                "name_conflicts",
                "RenamedRecordConflict"
            ),
            Ok("name_conflicts::RenamedRecordConflict".into()),
        );

        assert_eq!(
            run_resolve_item_value_namespace(
                &ir,
                &mut cache,
                "name_conflicts",
                "mod_fn_same_name::a_function"
            ),
            Ok("name_conflicts::mod_fn_same_name::a_function".into()),
        );
    }

    #[test]
    fn test_raw_ident() {
        // Test that we "unraw" idents before matching them by removing the `r#` prefix
        let mut ir = Ir::new_for_test(&["raw_idents"]);
        let mut cache = LookupCache::default();

        ir.add_udl_metadata(
            "raw_idents",
            vec![uniffi_meta::RecordMetadata {
                module_path: "raw_idents".into(),
                name: "Record".into(),
                orig_name: None,
                remote: false,
                fields: vec![],
                docstring: None,
            }
            .into()],
        )
        .unwrap();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "raw_idents", "r#Record"),
            Ok("raw_idents::Record".to_string()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "raw_idents", "r#Guid"),
            Ok("raw_idents::Guid".to_string()),
        );
    }

    #[test]
    fn test_same_item_imported_different_ways() {
        let ir = Ir::new_for_test(&["paths"]);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths", "Mod2Record"),
            Ok("paths::mod1::mod2::Mod2Record".to_string()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod5", "Mod2Record"),
            Ok("paths::mod1::mod2::Mod2Record".to_string()),
        );
    }
}
