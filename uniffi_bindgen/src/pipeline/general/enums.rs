/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(en: &mut Enum) -> Result<()> {
    en.is_flat = match en.shape {
        EnumShape::Error { flat } => flat,
        EnumShape::Enum => en.variants.iter().all(|v| v.fields.is_empty()),
    };
    Ok(())
}
