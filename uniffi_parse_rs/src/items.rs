/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use syn::{ext::IdentExt, Ident, ItemMacro, ItemType, LitStr, Path};

use crate::{CustomType, Enum, Function, Impl, Module, Object, Record, Trait, UseGlob, UseItem};

/// Item enum
///
/// This is a `syn::Item` that we've partially parsed.
/// The metadata module converts these into `uniffi::Metadata` items.
pub enum Item {
    Module(Module),
    Record(Record),
    Enum(Enum),
    Object(Object),
    Fn(Function),
    Impl(Impl),
    Trait(Trait),
    Type(ItemType),
    UseItem(UseItem),
    UseGlob(UseGlob),
    UseRemoteType(Path),
    IncludeScaffolding(LitStr),
    /// Macro expression that we haven't evaluated in any way yet.
    ///
    /// `macros::resolve_macros` inspects these macros and converts them to other variants like
    /// `UseRemoteType` if they match.
    Macro(ItemMacro),
    /// Custom type macro expression
    CustomType(CustomType),
    Udl(uniffi_meta::Type),
    /// Builtin items that we know about.
    Builtin(BuiltinItem),
}

/// Builtin item that we know about without needing to parse the source
///
/// We normally identify these by seeing a `use` statement that imports them from a crate that
/// we're not parsing the source of.
#[derive(Debug, Clone, Copy)]
pub enum BuiltinItem {
    UnitType,
    Boolean,
    String,
    Str,
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
    SystemTime,
    Duration,
    Vec,
    HashMap,
    Option,
    Result,
    Arc,
    Box,
    UniffiMacro(&'static str),
}

impl BuiltinItem {
    pub fn has_generic_args(&self) -> bool {
        matches!(
            self,
            Self::Vec | Self::HashMap | Self::Option | Self::Arc | Self::Box | Self::Result
        )
    }
}

impl Item {
    pub fn ident(&self) -> Option<Ident> {
        match &self {
            Item::Module(m) => Some(m.ident.unraw()),
            Item::Record(rec) => Some(rec.ident.unraw()),
            Item::Enum(en) => Some(en.ident.unraw()),
            Item::Object(o) => Some(o.ident.unraw()),
            Item::Fn(func) => Some(func.ident.unraw()),
            Item::Trait(tr) => Some(tr.ident.unraw()),
            Item::Type(ty) => Some(ty.ident.unraw()),
            Item::CustomType(c) => Some(c.ident.unraw()),
            Item::UseRemoteType(p) => p.segments.last().map(|s| s.ident.unraw()),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        match self.ident() {
            Some(i) => i.to_string(),
            None => match self {
                Item::Udl(ty) => ty.name().unwrap_or("<unnamed>").to_string(),
                _ => "<unnamed>".to_string(),
            },
        }
    }

    /// Is this a special item?
    ///
    /// "Special" here means that it's not a real Rust item, it's exists to give for UniFFI information
    /// about the interface
    pub fn is_special(&self) -> bool {
        match self {
            Item::Record(rec) => rec.attrs.remote,
            Item::Enum(en) => en.attrs.remote,
            Item::Object(o) => o.attrs.remote,
            Item::UseRemoteType(_) => true,
            Item::CustomType(_) => true,
            _ => false,
        }
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Module(m) => f.debug_tuple("Module").field(&m).finish(),
            Self::Record(rec) => f
                .debug_tuple("Record")
                .field(&rec.ident.to_string())
                .finish(),
            Self::Enum(en) => f.debug_tuple("Enum").field(&en.ident.to_string()).finish(),
            Self::Object(o) => f
                .debug_tuple("Interface")
                .field(&o.ident.to_string())
                .finish(),
            Self::Fn(func) => f.debug_tuple("Fn").field(&func.ident.to_string()).finish(),
            Self::Trait(tr) => f
                .debug_struct("Trait")
                .field("name", &tr.ident.to_string())
                .finish(),
            Self::Impl(imp) => f
                .debug_tuple("Impl")
                .field(&format!(
                    "<{} items>",
                    imp.constructors.len() + imp.methods.len()
                ))
                .finish(),
            Self::Type(ty) => f.debug_tuple("Type").field(&ty.ident.to_string()).finish(),
            Self::UseItem(use_item) => f
                .debug_tuple("UseItem")
                .field(&use_item.path)
                .field(&use_item.ident)
                .finish(),
            Self::UseGlob(use_glob) => f
                .debug_tuple("UseGlob")
                .field(&use_glob.module_path)
                .finish(),
            Self::UseRemoteType(_) => f.debug_tuple("UseRemoteType").finish(),
            Self::IncludeScaffolding(_) => f.debug_tuple("IncludeScaffolding").finish(),
            Self::Builtin(builtin) => f.debug_tuple("Builtin").field(&builtin).finish(),
            Self::CustomType(c) => f
                .debug_struct("CustomType")
                .field("ident", &c.ident.to_string())
                .finish(),
            Self::Udl(ty) => f.debug_tuple("Udl").field(ty).finish(),
            Self::Macro(_) => f.debug_tuple("Macro").finish(),
        }
    }
}
