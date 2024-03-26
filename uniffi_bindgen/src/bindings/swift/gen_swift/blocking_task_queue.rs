/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::CodeType;

#[derive(Debug)]
pub struct BlockingTaskQueueCodeType;

impl CodeType for BlockingTaskQueueCodeType {
    fn type_label(&self) -> String {
        // On Swift, we use a DispatchQueue for BlockingTaskQueue
        "DispatchQueue".into()
    }

    fn canonical_name(&self) -> String {
        "BlockingTaskQueue".into()
    }
}
