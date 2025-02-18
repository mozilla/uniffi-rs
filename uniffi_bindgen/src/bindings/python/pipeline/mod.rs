/* This Source Code Form is subject to the terms of the Mozilla Publicpypimod
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::pipeline::{general, initial, Pipeline};

// For now, this is just the general pipeline.
// Defining this allows us to use the pipeline CLI to inspect the general pipeline.
pub fn pipeline() -> Pipeline<initial::Root, general::Root> {
    general::pipeline()
}
