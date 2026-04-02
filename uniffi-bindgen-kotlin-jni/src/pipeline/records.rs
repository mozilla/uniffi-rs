/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_fields(fields: Vec<general::Field>, context: &Context) -> Result<Vec<Field>> {
    fields
        .into_iter()
        .enumerate()
        .map(|(i, f)| {
            Ok(Field {
                name: f.name,
                index: i,
                ty: f.ty.map_node(context)?,
                default: f.default.map_node(context)?,
                docstring: f.docstring,
            })
        })
        .collect()
}
