/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Lookup items for paths
//!
//! Also handles resolving types, which is closely related.

use core::fmt;
use std::collections::HashMap;

use proc_macro2::Span;
use syn::{ext::IdentExt, spanned::Spanned, Ident, ItemUse, Path, Token};

use crate::{
    files::FileId,
    use_::{parse_use, Use},
    BuiltinItem, Error,
    ErrorKind::*,
    Ir, Item, Module, Result,
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
    pub fn item(&self) -> &'ir Item {
        self.items
            .last()
            .expect("UniFFI internal error in Path::item(): items is empty")
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

    pub fn module_path(&self) -> Self {
        let mut path = self.clone();
        path.pop();
        path
    }

    pub fn module(&self) -> &'ir Module {
        self.items
            .iter()
            .rev()
            .find_map(|i| match i {
                Item::Module(m) => Some(m),
                _ => None,
            })
            .expect("UniFFI internal error in Path::module: no modules found")
    }

    pub fn crate_root(&self) -> &'ir Module {
        match self.items.first() {
            Some(Item::Module(m)) => m,
            i => panic!(
                "UniFFI internal error in Path::crate_root: first item is not a module ({i:?})"
            ),
        }
    }

    pub fn file_id(&self) -> FileId {
        self.module().source
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
    pub fn resolve(&self, ir: &'ir Ir, cache: &mut LookupCache<'ir>, path: &Path) -> Result<Self> {
        trace!(
            "Path::resolve {} -- {}",
            self.path_string(),
            quote::quote! { #path }
        );

        if path.leading_colon.is_some() || self.items.is_empty() {
            return self.resolve_global_path(ir, cache, path);
        }

        let mut current_path = self.clone();
        if path.segments.is_empty() {
            return Err(Error::new(self.file_id(), path.span(), NotFound));
        }
        // For the first segment only, try falling back to a global lookup on lookup errors
        let first_ident = &path.segments.first().unwrap().ident;
        match current_path.push_ident(ir, cache, first_ident) {
            Ok(()) => (),
            Err(e) if e.is_not_found() => {
                trace!("  PathError::NotFound, try global lookup");
                return self.resolve_global_path(ir, cache, path);
            }
            Err(e) => return Err(e),
        }

        for seg in path.segments.iter().skip(1) {
            trace!(
                "  push_ident (path: {}, ident: {})",
                current_path.path_string(),
                seg.ident
            );
            current_path.push_ident(ir, cache, &seg.ident)?;
        }
        trace!("  resolved: {}", current_path.path_string());
        Ok(current_path)
    }

    pub fn resolve_global_path(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &Path,
    ) -> Result<Self> {
        trace!("Path::resolve_global_path {}", quote::quote! { #path });
        if path.segments.is_empty() {
            return Err(Error::new(self.file_id(), path.span(), NotFound));
        }

        let first_ident = &path.segments.first().unwrap().ident;

        // Lookup UDL item from the crate root
        if path.segments.len() == 1 {
            if let Some(item) = self.crate_root().lookup_udl_item(first_ident) {
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
                for seg in path.segments.iter().skip(1) {
                    trace!(
                        "  push_ident (path: {}, ident: {})",
                        rpath.path_string(),
                        seg.ident
                    );
                    rpath.push_ident(ir, cache, &seg.ident)?;
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

    /// Push a new ident to this path
    ///
    /// Update self based on adding a new ident at the end of the path.  This usually means pushing
    /// an new element to `self.items`, but there are some special cases:
    ///
    /// * idents like `super` or `crate` result in removing items.
    /// * If an item comes from a `use` statement, then `self.items` will be replaced with the source of the `use`.
    ///
    /// `push_ident` also checks and updates the [LookupCache] for the current item/ident pair.
    fn push_ident(
        &mut self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        ident: &Ident,
    ) -> Result<()> {
        // Special case `super` and `crate`, no need to check the cache for this one.
        if ident == "super" {
            if self.len() <= 1 {
                return Err(Error::new(self.file_id(), ident.span(), SuperInvalid));
            }
            self.pop();
            return Ok(());
        } else if ident == "crate" {
            if self.is_empty() {
                return Err(Error::new(self.file_id(), ident.span(), CrateInvalid));
            }
            self.items.truncate(1);
            return Ok(());
        }

        let Item::Module(module) = self.item() else {
            // We currently assume non-module items have no child items
            return Err(Error::new(self.file_id(), ident.span(), NotFound));
        };
        let key = (module.id, ident.clone());
        match cache.0.get(&key) {
            Some(CacheEntry::Result(Ok(path))) => {
                *self = path.clone();
                return Ok(());
            }
            Some(CacheEntry::Result(Err(e))) => {
                return Err(e.clone());
            }
            Some(CacheEntry::Resolving) => {
                return Err(Error::new(self.file_id(), ident.span(), CycleDetected));
            }
            None => {
                cache.0.insert(key.clone(), CacheEntry::Resolving);
            }
        }
        let result = self._push_ident(ir, cache, module, ident);

        match result {
            Ok(()) => {
                cache.0.insert(key, CacheEntry::Result(Ok(self.clone())));
                Ok(())
            }
            Err(e) => {
                cache.0.insert(key, CacheEntry::Result(Err(e.clone())));
                Err(e)
            }
        }
    }

    /// Non-caching part of `_push_ident`.
    ///
    /// This implements the logic of `push_ident`, but doesn't handle the cache for this ident/item
    /// pair.  However, it still inputs the cache and uses it when resolving items from a `use`
    /// statement.
    fn _push_ident(
        &mut self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module: &'ir Module,
        ident: &Ident,
    ) -> Result<()> {
        let ident = ident.unraw();
        if self.push_ident_udl_item(module, &ident)? {
            return Ok(());
        }
        if self.push_ident_special_item(ir, cache, module, &ident)? {
            return Ok(());
        }
        let mut use_glob_paths = vec![];
        if self.push_ident_item_or_non_glob_use(ir, cache, module, &ident, &mut use_glob_paths)? {
            return Ok(());
        }
        if self.push_ident_glob_use(ir, cache, use_glob_paths)? {
            Ok(())
        } else {
            Err(Error::new(self.file_id(), ident.span(), NotFound))
        }
    }

    /// Push a UDL item for `push_ident` if possible
    fn push_ident_udl_item(&mut self, module: &'ir Module, ident: &Ident) -> Result<bool> {
        // First, see if there's a UDL item
        if let Some(item) = module.lookup_udl_item(ident) {
            self.push(item);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Push a "special" items like `uniffi::custom_type!` for push_ident, if possible.
    //
    // These don't represent real Rust items, they're more like instructions to UniFFI.
    fn push_ident_special_item(
        &mut self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module: &'ir Module,
        ident: &Ident,
    ) -> Result<bool> {
        let mut found = None;
        for item in module.items.iter().filter(|i| i.is_special()) {
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
            Some((_, item)) => {
                self.push_or_resolve(ir, cache, item)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    // Push a module child or an item from a (non-glob) use statement for push_ident, if possible.
    //
    // While we're looking for these items, we also push any use glob's we see to `use_glob_paths`
    fn push_ident_item_or_non_glob_use(
        &mut self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module: &'ir Module,
        ident: &Ident,
        use_glob_paths: &mut Vec<(Path, Token![*])>,
    ) -> Result<bool> {
        enum FoundItem<'ir> {
            Use(&'ir ItemUse, RPath<'ir>),
            Item(Ident, &'ir Item),
        }
        impl FoundItem<'_> {
            fn span(&self) -> Span {
                match self {
                    Self::Use(item_use, _) => item_use.span(),
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
            // Functions live in a different namespace than modules and types, which can lead to
            // incorrect NameConflict errors.  Luckily, the only reason we need to resolve paths is
            // to lookup types so we can just ignore skip any functions
            if matches!(item, Item::Fn(_)) {
                continue;
            }

            if let Some(item_ident) = item.ident() {
                if &item_ident == ident {
                    if let Some(found) = found {
                        return Err(Error::new(self.file_id(), item_ident.span(), NameConflict)
                            .context(self.file_id(), found.span(), "previous item"));
                    }
                    found = Some(FoundItem::Item(item_ident, item));
                }
            } else if let Item::Use(item_use) = item {
                match parse_use(self.file_id(), item_use, ident)? {
                    Use::Path(p) => {
                        let resolved = match self.resolve(ir, cache, &p) {
                            Ok(p) => p,
                            // ignore not found errors, maybe this is an item from another
                            // crate.
                            Err(e) if e.is_not_found() => continue,
                            Err(e) => {
                                return Err(e.context(
                                    self.file_id(),
                                    item_use.span(),
                                    "while resolving use",
                                ))
                            }
                        };
                        if let Some(found) = &found {
                            if found.matches_use(resolved) {
                                // If multiple use statements resolve to the same item, that's
                                // okay just skip the extra ones.
                                continue;
                            }
                            return Err(Error::new(self.file_id(), item_use.span(), NameConflict)
                                .context(self.file_id(), found.span(), "previous item"));
                        }
                        found = Some(FoundItem::Use(item_use, resolved));
                    }
                    Use::GlobPaths(paths) => {
                        // Not used now, but let's store for the next step
                        use_glob_paths.extend(paths);
                    }
                    Use::None => (),
                }
            }
        }
        match found {
            Some(FoundItem::Item(_, item)) => {
                self.push_or_resolve(ir, cache, item)?;
                Ok(true)
            }
            Some(FoundItem::Use(_, path)) => {
                *self = path.clone();
                Ok(true)
            }
            None => Ok(false),
        }
    }

    // Push an item from a glob use statement for push_ident, if possible.
    fn push_ident_glob_use(
        &mut self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        use_glob_paths: Vec<(Path, Token![*])>,
    ) -> Result<bool> {
        let mut found = None;

        for (glob_path, token) in use_glob_paths {
            let path = match self.resolve(ir, cache, &glob_path) {
                Ok(path) => path,
                Err(e) if e.is_not_found() || e.is_cycle_detected() => continue,
                Err(e) => {
                    return Err(e.context(self.file_id(), token.span(), "while resolving glob"))
                }
            };
            match &found {
                None => found = Some((path, token)),
                Some((prev_path, prev_token)) => {
                    if *prev_path != path {
                        return Err(Error::new(self.file_id(), token.span(), NameConflict)
                            .context(self.file_id(), prev_token.span(), "previous item"));
                    }
                }
            }
        }

        match found {
            None => Ok(false),
            Some((path, _)) => {
                *self = path;
                Ok(true)
            }
        }
    }

    fn push_or_resolve(
        &mut self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        item: &'ir Item,
    ) -> Result<()> {
        match item {
            Item::UseRemoteType(path) => {
                *self = self.resolve(ir, cache, path)?;
            }
            _ => {
                self.items.push(item);
            }
        }
        Ok(())
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
pub struct LookupCache<'ir>(HashMap<(usize, Ident), CacheEntry<'ir>>);

enum CacheEntry<'ir> {
    Result(Result<RPath<'ir>>),
    /// We're currently resolving this item, if we try to resolve this again that means there's a
    /// circular dependency somewhere
    Resolving,
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
            .resolve(ir, cache, &syn::parse_str(path).unwrap())
            .map(|path| format!("{path}"))
            .map_err(|e| e.kind)
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
