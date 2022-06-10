/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # FFI Names
//!
//! This module calculates the names we export in dylibs.
//!
//! We have two main goals with naming things:
//!   - Avoid name collisions.  For example when two objects having methods with the same name or two
//!     UniFFI crates are bundled together in the same dylib.
//!   - Guard against accidentally using foreign-language bindings generated from one version of an
//!     interface with the compiled Rust code from a different version of that interface.  We add
//!     checksums that depend on things like function signatures to avoid the possibility of the
//!     bindings calling into the scaffolding when they don't agree on the interface. The result
//!     will be an ugly inscrutable link-time error, but that is a lot better than triggering
//!     potentially arbitrary memory unsafety!

use super::{CallbackInterface, Constructor, Function, Method, Object};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// API contract version.  Bump this if you make any changes to how the foreign bindings calls the
/// scaffolding functions, for example the semantics of `RustCallStatus` or how types are
/// lifted/lowered.
const UNIFFI_API_VERSION: u32 = 0;

/// Generate a checksum to include in an interface component name
///
/// This is used to guard against accidentally using foreign-language bindings generated from one
/// version of an interface with the compiled Rust code from a different version of that interface.
///
/// This checksum is derived from the component data, which includes things like the function
/// signature.
///
/// Note that this is designed to prevent accidents, not attacks, so there is no need for the
/// checksum to be cryptographically secure.
fn checksum<T: Hash>(instance: &T) -> String {
    let mut hasher = DefaultHasher::new();
    instance.hash(&mut hasher);
    format!("{:x}", (hasher.finish() & 0x000000000000FFFF) as u16)
}

/// Calculates FFI names for the interface
///
/// This is an enum because we calculate our prefix differently depending on if we're using a UDL
/// file or a proc-macro.  It's is a bit unfortunate, but it should work out okay.
#[derive(Clone, Debug, Hash)]
pub enum FFINames {
    // Namespace of a UDL file
    InterfaceNamespace(String),
    // Name of the crate, this comes from the proc-macro code
    CrateName(String),
    // Module path of the item, including the crate name.  This is comes the proc-macro code when
    // run on nightly.  We don't currently use the extra parts yet, but maybe we could eventually
    // add support for nested modules in the bindings code.
    ModulePath(String),
}

impl FFINames {
    /// Get a prefix to use for the name of something
    ///
    /// This prefix includes:
    ///   - The word "uniffi" to avoid name collisions with other exports
    ///   - The UDL namespace, crate name, or full module path.  Which one we use depends on how
    ///     the scaffolding was generated.  The important thing is that different crates can't have
    ///     the same prefix. This avoids collisions in the case where multiple UniFFI crates are
    ///     bundled together in a single dylib.
    ///   - UNIFFI_API_VERSION to cause lookup/linker errors rather than UB when the scaffolding
    ///     API contract changes.
    ///   - A unique string for the kind of thing we're naming (function, method, constructor,
    ///     etc).  This prevents name collisions between different kinds of things, for if there's
    ///     an `Car.drive()` method and a `car_drive()` function.
    fn prefix(&self, kind: &str) -> String {
        let name_string = match self {
            Self::InterfaceNamespace(name) | Self::CrateName(name) | Self::ModulePath(name) => name,
        };
        format!("_uniffi_v{}_{}_{}", UNIFFI_API_VERSION, name_string, kind)
    }

    pub fn function(&self, func: &Function) -> String {
        format!("{}_{}_{}", self.prefix("func"), checksum(func), func.name)
    }

    pub fn constructor(&self, obj: &Object, cons: &Constructor) -> String {
        format!(
            "{}_{}_{}_{}",
            self.prefix("constructor"),
            checksum(cons),
            obj.name(),
            cons.name()
        )
    }

    pub fn method(&self, obj: &Object, meth: &Method) -> String {
        // Note: use the object checksum for this, since it includes the method signature but also the
        // rest of the object definition.
        format!(
            "{}_{}_{}_{}",
            self.prefix("methods"),
            checksum(obj),
            obj.name(),
            meth.name()
        )
    }

    pub fn object_free(&self, obj: &Object) -> String {
        format!("{}_{}_{}", self.prefix("free"), checksum(obj), obj.name())
    }

    pub fn callback_init(&self, callback: &CallbackInterface) -> String {
        format!(
            "{}_{}_{}_init",
            self.prefix("callback"),
            checksum(callback),
            callback.name()
        )
    }

    /// Name for a UniFFI global function, like `rustbuffer_alloc()`
    ///
    /// This name doesn't include a checksum.  If we make changes to those functions, then we need to
    /// bump the UNIFFI_API_VERSION.
    pub fn uniffi_func(&self, name: &str) -> String {
        format!("{}_{}", self.prefix("uniffi"), name)
    }
}
