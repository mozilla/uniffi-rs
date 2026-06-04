/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

impl UniffiTraitMethods {
    // Rust has 2 display traits, while Kotlin has one.
    // Prefer `Display` but use `Debug` otherwise
    pub fn to_string(&self) -> Option<&Method> {
        self.display_fmt.as_ref().or(self.debug_fmt.as_ref())
    }

    pub fn jni_methods(&self) -> impl Iterator<Item = (&str, JniMethodKind, &Callable)> + '_ {
        // We only need to generate one of `Display` or `Debug`
        let to_string = match (&self.display_fmt, &self.debug_fmt) {
            (Some(meth), _) => Some((meth, JniMethodKind::TraitMethodDisplayFmt)),
            (None, Some(meth)) => Some((meth, JniMethodKind::TraitMethodDebugFmt)),
            _ => None,
        };

        to_string
            .into_iter()
            .chain(
                self.eq_eq
                    .as_ref()
                    .map(|meth| (meth, JniMethodKind::TraitMethodEqEq)),
            )
            .chain(
                self.eq_ne
                    .as_ref()
                    .map(|meth| (meth, JniMethodKind::TraitMethodEqNe)),
            )
            .chain(
                self.hash_hash
                    .as_ref()
                    .map(|meth| (meth, JniMethodKind::TraitMethodHashHash)),
            )
            .chain(
                self.ord_cmp
                    .as_ref()
                    .map(|meth| (meth, JniMethodKind::TraitMethodOrdCmp)),
            )
            .map(|(meth, kind)| (meth.jni_method_name.as_str(), kind, &meth.callable))
    }
}
