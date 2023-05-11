/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeOracle, CodeType};

pub struct ForeignExecutorCodeType;

impl CodeType for ForeignExecutorCodeType {
    fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
        // Kotlin uses a CoroutineScope for ForeignExecutor
        "CoroutineScope".into()
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        "ForeignExecutor".into()
    }

    fn initialization_fn(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some("FfiConverterForeignExecutor.register".into())
    }
}
