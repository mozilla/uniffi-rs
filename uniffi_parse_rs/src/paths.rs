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
    files::FileId, BuiltinItem, CustomType, Error, ErrorKind::*, Ir, Item, ItemNames, Module,
    Result, UseGlob, UseItem,
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
                    BuiltinItem::HashSet => path.push_str("HashSet"),
                    BuiltinItem::Option => path.push_str("Option"),
                    BuiltinItem::Result => path.push_str("Result"),
                    BuiltinItem::UniffiMacro(name) | BuiltinItem::UniffiDerive(name) => {
                        path.push_str(name)
                    }
                    BuiltinItem::From => path.push_str("From"),
                    BuiltinItem::UnexpectedUniFFICallbackError => {
                        path.push_str("UnexpectedUniFFICallbackError")
                    }
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
                let mut rpath = RPath::new(crate_root.module_item());
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
                    // The path points into an unparsed crate.  A `custom_type!` may still cover it,
                    // if its underlying type aliases the same path.
                    if namespace == Namespace::Type {
                        if let Some(mapped) = cache.external_type_mapping(path) {
                            trace!("  resolved to type mapping: {mapped}");
                            return Ok(mapped);
                        }
                    }
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
        // Special case `self`, `super` and `crate`, no need to check the cache for these.
        if ident == "self" {
            // `self` refers to the current module (e.g. `use self::submodule::Item`,
            // where `self::` disambiguates a module from an external crate with the same name).
            return Ok(ChildItem {
                path: self.clone(),
                vis: Visibility::Public,
            });
        } else if ident == "super" {
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
            Some((_, Item::UseRemoteType(path))) => {
                match self.resolve(ir, cache, path, namespace) {
                    Ok(path) => Ok(Some(ChildItem {
                        path,
                        vis: Visibility::Public,
                    })),
                    // `use_remote_type!(implementing_crate::Type)` doesn't require `Type`
                    // to live at that path: the macro resolves `Type` in the invoking
                    // module's scope and only uses `implementing_crate` for its
                    // `UniFfiTag`.  Fall through to the module's regular items (e.g. a
                    // `type Decimal = rust_decimal::Decimal;` alias next to the macro
                    // invocation).
                    Err(e) if e.is_not_found() => Ok(None),
                    Err(e) => Err(e),
                }
            }
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
                    Self::Use(item_use, _) => item_use.span,
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
            // Special items are handled by `child_special_item` before this runs.  Seeing
            // them again here would report spurious name conflicts, e.g. between a
            // `use_remote_type!` invocation and the type alias it covers.
            if item.is_special() {
                continue;
            }
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
                        let path = &use_item.path;
                        trace!("  use item: {} ({})", use_item.ident, quote::quote! {#path});
                        let resolved = match self.resolve(ir, cache, path, namespace) {
                            Ok(p) => p,
                            // ignore not found errors and cycle errors, maybe we need to do a
                            // global lookup.
                            Err(e) if e.is_not_found() || e.is_cycle_detected() => continue,
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
            Self::Type => match item {
                Item::Module(_)
                | Item::Record(_)
                | Item::Enum(_)
                | Item::Object(_)
                | Item::Trait(_)
                | Item::CustomType(_)
                | Item::Udl(_)
                | Item::Type(_)
                | Item::UseRemoteType(_) => true,
                Item::Builtin(builtin) => !matches!(
                    builtin,
                    BuiltinItem::UniffiMacro(_) | BuiltinItem::UniffiDerive(_)
                ),
                _ => false,
            },
            Self::Value => matches!(item, Item::Fn(_)),
            Self::Macro => match item {
                Item::Builtin(builtin) => {
                    matches!(
                        builtin,
                        BuiltinItem::UniffiMacro(_) | BuiltinItem::UniffiDerive(_)
                    )
                }
                _ => true,
            },
            Self::NonUniffiType => matches!(item, Item::NonUniffi(_, _)),
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
pub struct LookupCache<'ir> {
    // Cached results for `RPath::child`
    // This maps module id / ident / namespaces to lookup results
    pub children: HashMap<(usize, Ident, Namespace), Result<ChildItem<'ir>>>,
    // Module children we're currently resolving, this is used to detect cycles
    pub children_resolving: HashSet<(usize, Ident, Namespace)>,
    // Cached results for `RPath::public_path_to_item()`
    // This maps path strings to lookup results
    pub public_paths: HashMap<String, Result<ItemNames>>,
    // Type mappings indexed by the type they cover
    type_mappings: TypeMappingRegistry<'ir>,
}

impl<'ir> LookupCache<'ir> {
    /// Create a lookup cache for a fully-resolved IR
    pub fn new(ir: &'ir Ir) -> Self {
        Self {
            type_mappings: TypeMappingRegistry::new(ir),
            ..Self::empty()
        }
    }

    /// A cache with an empty type mapping registry: lookups resolve as if no mappings
    /// were registered
    ///
    /// This is correct in exactly two places — while the IR is still being resolved (the
    /// items the registry maps don't exist yet), and for the registry build itself.
    /// Everything else should use [LookupCache::new].
    pub(crate) fn empty() -> Self {
        Self {
            children: HashMap::new(),
            children_resolving: HashSet::new(),
            public_paths: HashMap::new(),
            type_mappings: TypeMappingRegistry::default(),
        }
    }

    /// Look up the type mapping registered for the item at `item_path`
    ///
    /// `uniffi::custom_type!` is type-keyed in the macro flow: the generated FFI impls apply
    /// wherever the type is named, no matter where the macro was invoked.  When parsing sources
    /// we resolve names lexically, so a path can reach the underlying type (a type alias or a
    /// non-UniFFI type) without ever passing through the module that invoked `custom_type!`.
    /// This maps such items back to their custom type — or, for hand-written `TypeId` impls,
    /// to the builtin container the impl's `TYPE_ID_META` declares.
    pub fn registered_type_mapping(&self, item_path: &RPath<'ir>) -> Option<RPath<'ir>> {
        // Don't map custom types to themselves
        if matches!(item_path.item(), Ok(Item::CustomType(_))) {
            return None;
        }
        self.type_mappings
            .by_item
            .get(&item_path.path_string())
            .cloned()
    }

    /// Look up the type mapping registered for a path into an unparsed crate
    ///
    /// This handles custom types (and hand-written `TypeId` impls) whose underlying type
    /// comes from a non-UniFFI crate,
    /// e.g. `type Decimal = rust_decimal::Decimal;` + `uniffi::custom_type!(Decimal, String)`.
    /// Other modules can name the type as `use rust_decimal::Decimal`, which never resolves
    /// to an item since `rust_decimal` isn't parsed.
    pub fn external_type_mapping(&self, path: &Path) -> Option<RPath<'ir>> {
        let key = external_path_key(path)?;
        self.type_mappings.by_external_path.get(&key).cloned()
    }
}

/// Type mappings indexed by the Rust type they cover
///
/// A mapping's value is the item to resolve the covered type to: the `custom_type!`
/// item, or the builtin container declared by a hand-written `TypeId` impl.
#[derive(Default)]
struct TypeMappingRegistry<'ir> {
    /// Canonical item path (`RPath::path_string`) of the underlying item -> mapped item
    by_item: HashMap<String, RPath<'ir>>,
    /// Normalized textual path into an unparsed crate -> mapped item
    by_external_path: HashMap<String, RPath<'ir>>,
}

impl<'ir> TypeMappingRegistry<'ir> {
    fn new(ir: &'ir Ir) -> Self {
        // Building the registry resolves paths itself.  Use a scratch cache — its
        // registry is empty, so those resolutions see "nothing registered" — and discard
        // it afterwards, so nothing memoized under that assumption survives into the
        // caller's cache.
        let cache = &mut LookupCache::empty();
        let mut registry = Self::default();
        for (root_path, root_module) in ir.crate_roots_and_paths() {
            let mut stack = vec![(root_path, root_module)];
            while let Some((module_path, module)) = stack.pop() {
                for item in module.items.iter() {
                    match item {
                        Item::Module(m) => stack.push((module_path.append_child(item), m)),
                        Item::CustomType(custom_type) => registry.register_custom_type(
                            ir,
                            cache,
                            &module_path,
                            module,
                            item,
                            custom_type,
                        ),
                        Item::UnresolvedImpl(imp) => {
                            registry.register_type_id_impl(ir, cache, &module_path, module, imp)
                        }
                        _ => (),
                    }
                }
            }
        }
        registry
    }

    fn register_custom_type(
        &mut self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        module: &'ir Module,
        custom_type_item: &'ir Item,
        custom_type: &CustomType,
    ) {
        let custom_type_path = module_path.append_child(custom_type_item);
        // Find the Rust item the custom type ident names, ignoring special items
        // (the custom type itself shadows the name in its own module).
        let mut underlying =
            match module_path.underlying_item(ir, cache, module, &custom_type.ident.unraw()) {
                Some(Underlying::Item(path)) => path,
                Some(Underlying::External(path)) => {
                    if let Some(key) = external_path_key(&path) {
                        self.by_external_path.entry(key).or_insert(custom_type_path);
                    }
                    return;
                }
                // The custom type ident doesn't name anything else, so there's nothing to map
                // (name resolution finds the custom type directly in this case).
                None => return,
            };

        // Register the underlying item, following type alias chains so that references to
        // any point of the chain resolve to the custom type.
        let mut seen = HashSet::new();
        loop {
            let alias_target = match underlying.item() {
                // A generic alias can't correspond to a (non-generic) custom type
                Ok(Item::Type(alias)) if alias.generics.params.is_empty() => Some(&alias.ty),
                Ok(Item::NonUniffi(..)) => None,
                _ => return,
            };
            let key = underlying.path_string();
            if !seen.insert(key.clone()) {
                return; // alias cycle
            }
            self.by_item
                .entry(key)
                .or_insert_with(|| custom_type_path.clone());

            // Follow the alias target if it's a plain, non-generic path
            let Some(syn::Type::Path(target)) = alias_target.map(Box::as_ref) else {
                return;
            };
            if target.qself.is_some()
                || target
                    .path
                    .segments
                    .iter()
                    .any(|seg| !seg.arguments.is_none())
            {
                return;
            }
            let Ok(parent) = underlying.parent() else {
                return;
            };
            match parent.resolve_type_or_non_uniffi(ir, cache, &target.path) {
                Ok(next) => underlying = next,
                Err(e) if e.is_not_found() => {
                    if let Some(key) = external_path_key(&target.path) {
                        self.by_external_path.entry(key).or_insert(custom_type_path);
                    }
                    return;
                }
                Err(_) => return,
            }
        }
    }

    /// Register a hand-written `TypeId` impl for a generic container type
    ///
    /// Generic types can't be covered by `custom_type!`.  Instead, crates expose generic
    /// containers from non-UniFFI crates by hand-writing the FFI trait impls, and the
    /// `TYPE_ID_META` const in the `TypeId` impl declares the wire shape:
    ///
    /// ```ignore
    /// impl<T> TypeId<UniFfiTag> for IndexSet<T> {
    ///     const TYPE_ID_META: MetadataBuffer =
    ///         MetadataBuffer::from_code(metadata::codes::TYPE_HASH_SET).concat(T::TYPE_ID_META);
    /// }
    /// ```
    ///
    /// Parse that declaration and register the target type to resolve like the equivalent
    /// builtin container (`HashSet<T>` here).
    fn register_type_id_impl(
        &mut self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        module: &'ir Module,
        imp: &syn::ItemImpl,
    ) {
        // `impl<..> TypeId<..> for ..`
        let Some((None, trait_path, _)) = &imp.trait_ else {
            return;
        };
        let Some(trait_seg) = trait_path.segments.last() else {
            return;
        };
        if trait_seg.ident != "TypeId" {
            return;
        }
        // `const TYPE_ID_META: MetadataBuffer = ..;`
        let Some(expr) = imp.items.iter().find_map(|item| match item {
            syn::ImplItem::Const(c) if c.ident == "TYPE_ID_META" => Some(&c.expr),
            _ => None,
        }) else {
            return;
        };
        let Some((code, params)) = parse_type_id_meta_expr(expr) else {
            return;
        };
        let builtin: &'static Item = match (code.as_str(), params.len()) {
            ("TYPE_VEC", 1) => &Item::Builtin(BuiltinItem::Vec),
            ("TYPE_OPTION", 1) => &Item::Builtin(BuiltinItem::Option),
            ("TYPE_HASH_SET", 1) => &Item::Builtin(BuiltinItem::HashSet),
            ("TYPE_HASH_MAP", 2) => &Item::Builtin(BuiltinItem::HashMap),
            _ => return,
        };
        // The target type's generic arguments must be exactly the params concatenated into
        // `TYPE_ID_META`, in the same order — the builtin resolves its generics positionally.
        let syn::Type::Path(self_ty) = imp.self_ty.as_ref() else {
            return;
        };
        let Some(self_seg) = self_ty.path.segments.last() else {
            return;
        };
        let syn::PathArguments::AngleBracketed(args) = &self_seg.arguments else {
            return;
        };
        let arg_idents: Vec<&Ident> = args
            .args
            .iter()
            .filter_map(|arg| match arg {
                syn::GenericArgument::Type(syn::Type::Path(p)) => p.path.get_ident(),
                _ => None,
            })
            .collect();
        if arg_idents.len() != args.args.len()
            || arg_idents.len() != params.len()
            || arg_idents.iter().zip(&params).any(|(a, b)| **a != *b)
        {
            return;
        }
        // Register the target's base path, without the generic arguments
        let mut base_path = self_ty.path.clone();
        if let Some(seg) = base_path.segments.last_mut() {
            seg.arguments = syn::PathArguments::None;
        }
        let underlying = if let Some(ident) = base_path.get_ident() {
            module_path.underlying_item(ir, cache, module, &ident.unraw())
        } else {
            match module_path.resolve_type_or_non_uniffi(ir, cache, &base_path) {
                Ok(path) => Some(Underlying::Item(path)),
                Err(e) if e.is_not_found() => Some(Underlying::External(base_path)),
                Err(_) => None,
            }
        };
        match underlying {
            Some(Underlying::Item(path)) => {
                self.by_item
                    .entry(path.path_string())
                    .or_insert_with(|| RPath::new(builtin));
            }
            Some(Underlying::External(path)) => {
                if let Some(key) = external_path_key(&path) {
                    self.by_external_path
                        .entry(key)
                        .or_insert_with(|| RPath::new(builtin));
                }
            }
            None => (),
        }
    }
}

enum Underlying<'ir> {
    /// The custom type covers an item we parsed
    Item(RPath<'ir>),
    /// The custom type covers a path into an unparsed crate
    External(Path),
}

/// Resolution helpers for building the type mapping registry
impl<'ir> RPath<'ir> {
    /// Find the non-special item a custom type's ident names in this module
    fn underlying_item(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module: &'ir Module,
        ident: &Ident,
    ) -> Option<Underlying<'ir>> {
        let mut use_globs = vec![];
        for item in module.items.iter() {
            if item.is_special() {
                continue;
            }
            match item {
                Item::Type(_) | Item::NonUniffi(..) if item.ident().as_ref() == Some(ident) => {
                    return Some(Underlying::Item(self.append_child(item)));
                }
                Item::UseItem(use_item) if use_item.ident == *ident => {
                    return match self.resolve_type_or_non_uniffi(ir, cache, &use_item.path) {
                        Ok(path) => Some(Underlying::Item(path)),
                        Err(e) if e.is_not_found() => {
                            Some(Underlying::External(use_item.path.clone()))
                        }
                        Err(_) => None,
                    };
                }
                Item::UseGlob(use_glob) => use_globs.push(use_glob),
                _ => (),
            }
        }
        for namespace in [Namespace::Type, Namespace::NonUniffiType] {
            if let Ok(Some(child)) =
                self.child_glob_use(ir, cache, ident, use_globs.clone(), namespace)
            {
                return Some(Underlying::Item(child.path));
            }
        }
        None
    }

    /// Resolve a path in the type namespace, falling back to non-UniFFI types
    fn resolve_type_or_non_uniffi(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &Path,
    ) -> Result<RPath<'ir>> {
        match self.resolve(ir, cache, path, Namespace::Type) {
            Err(e) if e.is_not_found() => self.resolve(ir, cache, path, Namespace::NonUniffiType),
            result => result,
        }
    }
}

/// Parse a `TYPE_ID_META` expression of the form
/// `MetadataBuffer::from_code(metadata::codes::TYPE_X).concat(T::TYPE_ID_META)..`
///
/// Returns the metadata code name and the generic params concatenated after it,
/// in concatenation order.
fn parse_type_id_meta_expr(mut expr: &syn::Expr) -> Option<(String, Vec<Ident>)> {
    let mut params = vec![];
    loop {
        match expr {
            // `<receiver>.concat(P::TYPE_ID_META)`
            syn::Expr::MethodCall(call) if call.method == "concat" && call.args.len() == 1 => {
                let syn::Expr::Path(arg) = &call.args[0] else {
                    return None;
                };
                let mut segments = arg.path.segments.iter();
                let (Some(param), Some(meta), None) =
                    (segments.next(), segments.next(), segments.next())
                else {
                    return None;
                };
                if meta.ident != "TYPE_ID_META" {
                    return None;
                }
                params.push(param.ident.clone());
                expr = &call.receiver;
            }
            // `MetadataBuffer::from_code(metadata::codes::TYPE_X)`
            syn::Expr::Call(call) if call.args.len() == 1 => {
                let syn::Expr::Path(func) = call.func.as_ref() else {
                    return None;
                };
                if func.path.segments.last()?.ident != "from_code" {
                    return None;
                }
                let syn::Expr::Path(code) = &call.args[0] else {
                    return None;
                };
                let code = code.path.segments.last()?.ident.to_string();
                // Params were collected outermost-in; concatenation order is base-out
                params.reverse();
                return Some((code, params));
            }
            _ => return None,
        }
    }
}

/// Normalized key for a path that should point into an unparsed crate
///
/// The key is built from the segment idents only.  Generic arguments are allowed on the
/// last segment (e.g. a field spelled `indexmap::IndexSet<Counter>`) — they're resolved
/// separately, according to the item the path maps to.
fn external_path_key(path: &Path) -> Option<String> {
    if path.segments.len() < 2 {
        return None;
    }
    let segments = path.segments.iter().collect::<Vec<_>>();
    let (_, non_last_segments) = segments.split_last()?;
    if non_last_segments.iter().any(|seg| !seg.arguments.is_none()) {
        return None;
    }
    Some(
        path.segments
            .iter()
            .map(|seg| seg.ident.unraw().to_string())
            .collect::<Vec<_>>()
            .join("::"),
    )
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
        "Result" | "std::result::Result" => Some(&Item::Builtin(BuiltinItem::Result)),
        "String" | "std::string::String" => Some(&Item::Builtin(BuiltinItem::String)),
        "str" | "std::primitive::str" => Some(&Item::Builtin(BuiltinItem::Str)),
        "From" | "std::convert::From" => Some(&Item::Builtin(BuiltinItem::From)),
        "std::collections::HashMap" => Some(&Item::Builtin(BuiltinItem::HashMap)),
        "std::collections::HashSet" => Some(&Item::Builtin(BuiltinItem::HashSet)),
        "std::sync::Arc" => Some(&Item::Builtin(BuiltinItem::Arc)),
        "std::time::SystemTime" => Some(&Item::Builtin(BuiltinItem::SystemTime)),
        "std::time::Duration" => Some(&Item::Builtin(BuiltinItem::Duration)),
        "uniffi::custom_type" => Some(&Item::Builtin(BuiltinItem::UniffiMacro("custom_type"))),
        "uniffi::custom_newtype" => {
            Some(&Item::Builtin(BuiltinItem::UniffiMacro("custom_newtype")))
        }
        "uniffi::export" => Some(&Item::Builtin(BuiltinItem::UniffiMacro("export"))),
        "uniffi::remote" => Some(&Item::Builtin(BuiltinItem::UniffiMacro("remote"))),
        "uniffi::use_remote_type" => {
            Some(&Item::Builtin(BuiltinItem::UniffiMacro("use_remote_type")))
        }
        "uniffi::Record" => Some(&Item::Builtin(BuiltinItem::UniffiDerive("Record"))),
        "uniffi::Enum" => Some(&Item::Builtin(BuiltinItem::UniffiDerive("Enum"))),
        "uniffi::Error" => Some(&Item::Builtin(BuiltinItem::UniffiDerive("Error"))),
        "uniffi::Object" => Some(&Item::Builtin(BuiltinItem::UniffiDerive("Object"))),
        "uniffi::UnexpectedUniFFICallbackError" => {
            Some(&Item::Builtin(BuiltinItem::UnexpectedUniFFICallbackError))
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
            .inspect(|path| println!("run_resolve_item path: {path:?}"))
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
        let crate_root = ir
            .crate_roots
            .get(&format_ident!("{crate_name}"))
            .unwrap_or_else(|| panic!("crate root not found: {crate_name}"));
        let mut path = RPath::new(crate_root.module_item());
        let mut module = crate_root.module();
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
        let mut cache = LookupCache::new(&ir);

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
        let mut cache = LookupCache::new(&ir);

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
        let mut cache = LookupCache::new(&ir);

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths", "mod3::Mod3Record"),
            Ok("paths::mod1::mod2::mod3::Mod3Record".to_string()),
        );

        // Leading `self::` refers to the current module
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "self::Mod1Record"),
            Ok("paths::mod1::Mod1Record".to_string()),
        );
        assert_eq!(
            run_resolve_item(
                &ir,
                &mut cache,
                "paths::mod1",
                "self::mod2::mod3::Mod3Record"
            ),
            Ok("paths::mod1::mod2::mod3::Mod3Record".to_string()),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod1", "self::missing::MyRecord"),
            Err(ErrorKind::NotFound),
        );

        // Named import through `self::` (`use self::inner::SelfUseRecord as ...` in mod6)
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod6", "SelfUseRecordRenamed"),
            Ok("paths::mod6::inner::SelfUseRecord".to_string()),
        );
        // Glob re-export through `self::` (`pub use self::inner::*;` in mod6)
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths::mod6", "SelfGlobRecord"),
            Ok("paths::mod6::inner::SelfGlobRecord".to_string()),
        );
        // ... and the re-exported item is visible from outside the module
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "paths", "mod6::SelfGlobRecord"),
            Ok("paths::mod6::inner::SelfGlobRecord".to_string()),
        );
    }

    #[test]
    fn test_use_remote_type_falls_through_to_local_items() {
        let ir = Ir::new_for_test(&["paths", "paths2", "paths3"]);
        let mut cache = LookupCache::new(&ir);

        // `use_remote_type!(paths3::ExternalRemote)` names a path that doesn't resolve
        // (`paths3` has no `ExternalRemote`).  Per the macro's semantics the type is
        // resolved in the invoking module's scope, so resolution must fall through to
        // the local alias instead of failing.
        assert_eq!(
            run_resolve_item(
                &ir,
                &mut cache,
                "paths::remote_fallthrough",
                "ExternalRemote"
            ),
            Ok("paths::remote_fallthrough::ExternalRemote".to_string()),
        );
    }

    #[test]
    fn test_resolve_item_with_crate_keyword() {
        let ir = Ir::new_for_test(&["paths"]);
        let mut cache = LookupCache::new(&ir);

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
        let mut cache = LookupCache::new(&ir);

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
        let mut cache = LookupCache::new(&ir);

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
        let mut cache = LookupCache::new(&ir);

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
        let mut cache = LookupCache::new(&ir);

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
            Err(ErrorKind::NotFound),
        );
    }

    #[test]
    fn test_name_conflicts() {
        let ir = Ir::new_for_test(&["name_conflicts"]);
        let mut cache = LookupCache::new(&ir);

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
            Ok("name_conflicts::CustomTypeConflict".into()),
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
        let mut cache = LookupCache::new(&ir);
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
        let mut cache = LookupCache::new(&ir);

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
