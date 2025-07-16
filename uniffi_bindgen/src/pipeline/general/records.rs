/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(rec: &mut Record) -> Result<()> {
    rec.fields_kind = if rec.fields.is_empty() {
        FieldsKind::Unit
    } else if rec.fields.iter().any(|f| f.name.is_empty()) {
        FieldsKind::Unnamed
    } else {
        FieldsKind::Named
    };
    UniffiTraitMethods::fill_from(&mut rec.uniffi_trait_methods, &rec.uniffi_traits);
    Ok(())
}
