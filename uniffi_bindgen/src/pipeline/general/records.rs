/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn fields_kind(fields: &[initial::Field]) -> FieldsKind {
    if fields.is_empty() {
        FieldsKind::Unit
    } else if fields.iter().any(|f| f.name.is_empty()) {
        FieldsKind::Unnamed
    } else {
        FieldsKind::Named
    }
}
