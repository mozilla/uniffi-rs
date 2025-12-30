/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    any::{type_name, Any},
    io::Write,
    marker::PhantomData,
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use heck::{ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};

use super::{MapNode, Node};

/// Bindgen pipeline
///
/// Input and Output are root nodes of the input/output IR.  This pipeline converts from `Input` to
/// `Output` using a series of passes.
///
/// See https://mozilla.github.io/uniffi-rs/latest/internals/bindings_ir_pipeline.html for details on
/// how this works.
pub struct Pipeline<Input, Output> {
    passes: Vec<Pass>,
    input: PhantomData<Input>,
    output: PhantomData<Output>,
}

/// A pipeline pass is a function that converts the root node of an IR into another root node
pub struct Pass {
    name: String,
    func: PassFn,
}

type PassFn = Box<dyn FnMut(Box<dyn Node>) -> Result<Box<dyn Node>>>;

pub fn new_pipeline<Input>() -> Pipeline<Input, Input> {
    Pipeline {
        passes: vec![],
        input: PhantomData,
        output: PhantomData,
    }
}

impl<Input, Output> Pipeline<Input, Output>
where
    Input: Node,
    Output: Node,
{
    /// Add a pass that mutates nodes in the current IR
    ///
    /// This uses [Node::visit_mut] to find all nodes of a given type, then passes those nodes to
    /// the provided closure to mutate them.
    pub fn pass<NewOutput, Context>(self, context: Context) -> Pipeline<Input, NewOutput>
    where
        Output: MapNode<NewOutput, Context> + Any,
        NewOutput: Node,
        Context: 'static,
    {
        let mut passes = self.passes;
        passes.push(Pass {
            name: type_name::<NewOutput>().to_string(),
            func: Box::new(move |root| {
                let output = root
                    .to_box_any()
                    .downcast::<Output>()
                    .map_err(|_| anyhow!("Error casting root node to {}", type_name::<Output>()))?;
                Ok(output.map_node(&context)?)
            }),
        });
        Pipeline {
            passes,
            input: PhantomData,
            output: PhantomData,
        }
    }

    /// Execute the pipeline
    pub fn execute(&mut self, root: Input) -> Result<Output> {
        self.execute_all_passes(root, &mut NullPipelineRecorder)
    }

    /// Execute the pipeline, printing out debugging information for each pass
    ///
    /// This is used to implement the `pipeline` CLI subcommand
    pub fn print_passes(&mut self, root: Input, opts: PrintOptions) -> Result<()> {
        let mut last_output: Option<(tempfile::TempPath, String)> = None;
        let mut recorder = PipelineCliRecorder::new(opts.clone());
        let execute_result = self.execute_all_passes(root, &mut recorder);

        let count = recorder.passes.len();

        for (i, (title, content)) in recorder.passes.into_iter().enumerate() {
            // Save output for diffing
            let mut output = tempfile::NamedTempFile::new()?;
            write!(output, "{content}")?;
            let output_path = output.into_temp_path();

            if opts.matches_pass(&title, i + 1 == count) {
                match (last_output, opts.no_diff) {
                    (None, _) | (Some(_), true) => {
                        // First pass, print out the content
                        let title = format!(" {title} ");
                        println!("{title:=^78}");
                        println!("{content}");
                    }
                    (Some((last_output, last_title)), _) => {
                        // Middle pass, print out the diff from the last run
                        Command::new("diff")
                            .args(["-du", "--color=auto"])
                            .arg(&last_output)
                            .arg(&output_path)
                            .arg("--label")
                            .arg(&last_title)
                            .arg("--label")
                            .arg(&title)
                            .spawn()?
                            .wait()?;
                    }
                }
                println!();
            }
            last_output = Some((output_path, title));
        }
        // Check the result after printing all passes.  This gives the user more context when things
        // go wrong.
        execute_result?;
        if matches!(opts.pass.as_deref(), None | Some("final")) {
            if let Some((output_path, _)) = last_output {
                println!("{:=^78}", " final ");
                println!("{}", std::fs::read_to_string(output_path)?);
            }
        }
        Ok(())
    }

    /// Execute each pass in the pipeline and convert `Self::Input` to `Self::Output`
    ///
    /// After each pass, call `recorder.report_pass`, passing it the name of the pass and the root node
    /// after the pass.
    fn execute_all_passes(
        &mut self,
        root: Input,
        recorder: &mut dyn PipelineRecorder,
    ) -> Result<Output> {
        recorder.record_pass("initial", &root);
        let mut root: Box<dyn Node> = Box::new(root);
        for pass in self.passes.iter_mut() {
            root = (pass.func)(root).with_context(|| format!("pass: {}", pass.name))?;
            recorder.record_pass(&pass.name, &*root);
        }
        let root = root
            .to_box_any()
            .downcast::<Output>()
            .map_err(|_| anyhow!("Output type mismatch"))?;
        Ok(*root)
    }
}

/// Records passes taken in a IR pipeline
pub trait PipelineRecorder {
    /// Record the result of a pass for the pipeline CLI
    fn record_pass(&mut self, name: &str, node: &dyn Node);
}

/// Implements PipelineRecorder by doing nothing.  This is what's used when we want to just
/// generate bindings, not print out the passes for the pipeline CLI
struct NullPipelineRecorder;

impl PipelineRecorder for NullPipelineRecorder {
    fn record_pass(&mut self, _name: &str, _node: &dyn Node) {}
}

/// Implements PipelineRecorder for the pipeline CLI
struct PipelineCliRecorder {
    opts: PrintOptions,
    passes: Vec<(String, String)>,
}

impl PipelineCliRecorder {
    fn new(opts: PrintOptions) -> Self {
        Self {
            opts,
            passes: vec![],
        }
    }
}

impl PipelineRecorder for PipelineCliRecorder {
    fn record_pass(&mut self, name: &str, node: &dyn Node) {
        self.passes
            .push((name.to_string(), pass_content(node, &self.opts)));
    }
}

#[derive(Clone)]
pub struct PrintOptions {
    pub pass: Option<String>,
    pub no_diff: bool,
    pub filter_type: Option<String>,
    pub filter_name: Option<String>,
}

impl PrintOptions {
    fn matches_pass(&self, title: &str, last: bool) -> bool {
        match self.pass.as_deref() {
            None => true,
            Some("last") => last,
            Some(p) => title.contains(p),
        }
    }

    fn has_filter(&self) -> bool {
        self.filter_type.is_some() || self.filter_name.is_some()
    }

    fn matches_node(&self, node: &dyn Node, child: &dyn Node) -> bool {
        if let Some(filter_type) = &self.filter_type {
            match node.type_name() {
                Some(name) if name.to_lowercase() == filter_type.to_lowercase() => (),
                _ => return false,
            }
        }
        if let Some(filter_name) = &self.filter_name {
            let Some(string_value) = child.as_any().downcast_ref::<String>() else {
                return false;
            };
            if !(string_value.contains(filter_name)
                || string_value.to_snake_case().contains(filter_name)
                || string_value.to_shouty_snake_case().contains(filter_name)
                || string_value.to_lower_camel_case().contains(filter_name)
                || string_value.to_upper_camel_case().contains(filter_name))
            {
                return false;
            }
        }
        true
    }
}

fn pass_content(node: &dyn Node, opts: &PrintOptions) -> String {
    if !opts.has_filter() {
        return format!("{node:#?}");
    }
    let mut search = NodeFilterSearch::new(opts);
    search.search(node);
    if search.results.is_empty() {
        return "Empty".to_string();
    }
    let mut content = String::new();
    for node_content in search.results {
        content.push_str(&format!("{node_content}\n"));
    }
    content
}

// Implements the depth-first-search to handle `pass_content` with a filter
struct NodeFilterSearch<'a> {
    opts: &'a PrintOptions,
    results: Vec<String>,
}

impl<'a> NodeFilterSearch<'a> {
    fn new(opts: &'a PrintOptions) -> Self {
        Self {
            opts,
            results: vec![],
        }
    }

    fn search(&mut self, node: &dyn Node) {
        // If any child nodes match, then add this node to the results
        let mut child_match = false;
        node.visit_children(&mut |child| {
            child_match = child_match || self.opts.matches_node(node, child);
        });
        if child_match {
            self.results.push(format!("{node:#?}"));
        } else {
            // Otherwise, continue recursing
            node.visit_children(&mut |child| self.search(child));
        }
    }
}
