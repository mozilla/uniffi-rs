/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::export::{ExportItem, Signature};

pub(super) fn gen_fn_metadata(sig: syn::Signature) -> syn::Result<ExportItem> {
    let sig = Signature::new(sig)?;
    Ok(ExportItem::Function { sig })
}
