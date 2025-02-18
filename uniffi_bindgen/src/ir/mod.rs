/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod general;
pub mod initial;
mod node;
mod pipeline;

pub use anyhow::{bail, Result};
pub use initial::Root as InitialRoot;
pub use node::{FromNode, IntoNode, Node};
pub use pipeline::{InitialPipeline, Pass, Pipeline, PrintStepsOptions, Step};
pub use uniffi_internal_macros::{construct_node, define_ir_pass, ir, ir_pass, Node};

/// General IR pipeline
///
/// This is the shared begining for all bindings pipelines.
/// Bindings generators will add language-specific passes to this.
pub fn general_pipeline(root: InitialRoot) -> impl Pipeline<Output = general::Root> {
    InitialPipeline::new(root).pass(general::pass::pass())
}
