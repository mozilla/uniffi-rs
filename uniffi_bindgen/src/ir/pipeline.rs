/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{fs, io::Write, marker::PhantomData, process::Command};

use anyhow::{Context, Result};
use heck::{ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};

use super::{FromNode, InitialRoot, IntoNode, Node};

/// Pipeline that converts `InitialRoot` into type T, which will be something like `python::pipeline::Root`.
///
/// Pipelines consist of a series of passes, which can be further broken down into a series of
/// steps.
pub trait Pipeline {
    type Output: Node;

    /// Execute each step in the pipeline and convert InitialRoot to `Self::Output`
    ///
    /// After each step, call `report_step`, passing it the name of the step and the root node
    /// after the pass.
    fn execute_all_steps(
        &mut self,
        recorder: &mut dyn PipelineStepRecorder,
    ) -> Result<Self::Output>;

    /// Execute the pipeline, without printing out any details
    ///
    /// This is used when we just want to generate the bindings and not print out the pipeline.
    fn execute(&mut self) -> Result<Self::Output> {
        self.execute_all_steps(&mut NullPipelineStepRecorder)
    }

    /// Add a new pass to the pipeline
    ///
    /// Pass in a IR pipeline
    ///
    /// This will transform the IR from `PrevRoot` to `NextRoot`.
    ///
    /// Passes consist of a series of steps, which logically add and remove fields.  However, to keep
    /// the types simple and compile times low we don't generate a full set of IR types for each step.
    /// Instead, a single "Pass IR" is defined which is what the steps operate on.  The Pass IR
    /// contains both added and removed fields/variants.  Steps are responsible for populating any
    /// added field and transforming enums so they don't contain removed variants.  See the IR internal
    /// doc for details.
    fn pass<PassRoot, NextRoot>(self, pass: Pass<PassRoot>) -> impl Pipeline<Output = NextRoot>
    where
        Self: Sized,
        PassRoot: Node + FromNode<Self::Output>,
        NextRoot: Node + FromNode<PassRoot>,
    {
        MappedPipeline {
            prev: self,
            pass,
            next_root: PhantomData,
        }
    }

    /// Execute the pipeline and print the root value at each step
    fn print_steps(&mut self, opts: PrintStepsOptions) -> Result<()> {
        let mut last_output: Option<(tempfile::TempPath, String)> = None;
        let mut step_recorder = PipelineCliStepRecorder::new(opts.clone());
        self.execute_all_steps(&mut step_recorder)?;

        let step_count = step_recorder.steps.len();

        for (i, (title, content)) in step_recorder.steps.into_iter().enumerate() {
            // Save output for diffing
            let mut output = tempfile::NamedTempFile::new()?;
            write!(output, "{content}")?;
            let output_path = output.into_temp_path();

            if opts.matches_step(&title, i == step_count - 2) {
                match last_output {
                    None => {
                        // First pass, print out the content
                        let title = format!(" {title} ");
                        println!("{title:=^78}");
                        println!("{content}");
                    }
                    Some((last_output, last_title)) => {
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
        // Last pass, print out the entire content from the last run
        if opts.steps.is_none() {
            if let Some((last_output, _)) = last_output {
                println!("{:=^78}", " final ");
                println!("{}", fs::read_to_string(last_output)?)
            }
        }

        Ok(())
    }
}

/// Pipeline pass
pub struct Pass<PassRoot> {
    pub name: &'static str,
    pub steps: Vec<Step<PassRoot>>,
}

impl<PassRoot> Pass<PassRoot> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            steps: vec![],
        }
    }

    /// Add a step to a pass
    ///
    /// `func` is a function that inputs `&mut Node` for any node type and returns
    /// `anyhow::Result<()>`.  Pass steps typically add new fields to the IR by setting values for
    /// fields that were previously empty.  Alternatively, passes can simply mutate the nodes, for
    /// example `general::sort::step` sorts IR definitions so type dependencies come before the
    /// types that depend on them.
    pub fn step<F, N>(mut self, func: F) -> Self
    where
        F: Fn(&mut N) -> Result<()> + 'static,
        N: Node,
        PassRoot: Node,
    {
        self.steps.push(Step::new(
            std::any::type_name_of_val(&func),
            move |root: &mut PassRoot| root.try_visit_mut(&func),
        ));
        self
    }

    fn execute<PrevRoot, NextRoot>(
        &self,
        root: PrevRoot,
        recorder: &mut dyn PipelineStepRecorder,
    ) -> Result<NextRoot>
    where
        PrevRoot: Node,
        PassRoot: Node + FromNode<PrevRoot>,
        NextRoot: Node + FromNode<PassRoot>,
    {
        let mut root = root.into_node()?;
        recorder.record_step(&format!("{} (from prev IR)", self.name), &root);
        for step in &self.steps {
            (step.func)(&mut root)
                .with_context(|| format!("While executing step {}", step.name))?;
            recorder.record_step(step.name, &root);
        }
        let root = root.into_node()?;
        recorder.record_step(&format!("{} (into next IR)", self.name), &root);
        Ok(root)
    }
}

/// Single step in a pass, see `Pipeline::pass` for how this is used.
pub struct Step<PassRoot> {
    name: &'static str,
    /// Function responsible for the step.
    ///
    /// Logically, these functions will add/remove fields and variants, however no types are
    /// actually changing.  Instead, they just mutate the nodes and add data to empty fields,
    /// convert removed variants into other variants, etc.
    #[allow(clippy::type_complexity)]
    func: Box<dyn Fn(&mut PassRoot) -> Result<()>>,
}

impl<PassRoot> Step<PassRoot> {
    pub fn new(name: &'static str, func: impl Fn(&mut PassRoot) -> Result<()> + 'static) -> Self {
        Self {
            name,
            func: Box::new(func),
        }
    }
}

struct MappedPipeline<PrevPipeline, PassRoot, NextRoot> {
    prev: PrevPipeline,
    pass: Pass<PassRoot>,
    next_root: PhantomData<NextRoot>,
}

impl<PrevPipeline, PassRoot, NextRoot> Pipeline for MappedPipeline<PrevPipeline, PassRoot, NextRoot>
where
    PrevPipeline: Pipeline,
    PassRoot: Node + FromNode<PrevPipeline::Output>,
    NextRoot: Node + FromNode<PassRoot>,
{
    type Output = NextRoot;

    fn execute_all_steps(&mut self, recorder: &mut dyn PipelineStepRecorder) -> Result<NextRoot> {
        let prev_node = self.prev.execute_all_steps(recorder)?;
        let next_node = self.pass.execute(prev_node, recorder)?;
        Ok(next_node)
    }
}

/// Records steps taken in a IR pipeline
pub trait PipelineStepRecorder {
    /// Record the result of a step for the pipeline CLI
    fn record_step(&mut self, name: &str, node: &dyn Node);
}

/// Implements PipelineStepRecorder by doing nothing.  This is what's used when we want to just
/// generate bindings, not print out the steps for the pipeline CLI
struct NullPipelineStepRecorder;

impl PipelineStepRecorder for NullPipelineStepRecorder {
    fn record_step(&mut self, _name: &str, _node: &dyn Node) {}
}

/// Implements PipelineStepRecorder for the pipeline CLI
struct PipelineCliStepRecorder {
    opts: PrintStepsOptions,
    steps: Vec<(String, String)>,
}

impl PipelineCliStepRecorder {
    fn new(opts: PrintStepsOptions) -> Self {
        Self {
            opts,
            steps: vec![],
        }
    }
}

impl PipelineStepRecorder for PipelineCliStepRecorder {
    fn record_step(&mut self, name: &str, node: &dyn Node) {
        self.steps
            .push((name.to_string(), step_content(node, &self.opts)));
    }
}

/// Initial pipeline that just outputs `InitialRoot`
pub struct InitialPipeline {
    root: InitialRoot,
}

impl InitialPipeline {
    pub fn new(root: InitialRoot) -> Self {
        Self { root }
    }
}

impl Pipeline for InitialPipeline {
    type Output = InitialRoot;

    fn execute_all_steps(
        &mut self,
        recorder: &mut dyn PipelineStepRecorder,
    ) -> Result<Self::Output> {
        recorder.record_step("initial", &self.root);
        Ok(self.root.clone())
    }
}

#[derive(Clone)]
pub struct PrintStepsOptions {
    pub steps: Option<String>,
    pub filter_type: Option<String>,
    pub filter_name: Option<String>,
}

impl PrintStepsOptions {
    fn matches_step(&self, step_title: &str, last_step: bool) -> bool {
        match self.steps.as_deref() {
            None => true,
            Some("last") => last_step,
            Some(p) => step_title.contains(p),
        }
    }

    fn has_filter(&self) -> bool {
        self.filter_type.is_some() || self.filter_name.is_some()
    }

    fn matches_node(&self, node: &dyn Node, child: &dyn Node) -> bool {
        if let Some(filter_type) = &self.filter_type {
            if node.type_name() != Some(filter_type) {
                return false;
            }
        }
        if let Some(filter_name) = &self.filter_name {
            let Some(string_value) = child.as_any().downcast_ref::<String>() else {
                return false;
            };
            if !(string_value == filter_name
                || string_value.to_snake_case() == *filter_name
                || string_value.to_shouty_snake_case() == *filter_name
                || string_value.to_lower_camel_case() == *filter_name
                || string_value.to_upper_camel_case() == *filter_name)
            {
                return false;
            }
        }
        true
    }
}

fn step_content(node: &dyn Node, opts: &PrintStepsOptions) -> String {
    if !opts.has_filter() {
        return format!("{node:#?}");
    }
    let mut search = NodeFilterSearch::new(opts);
    search.search(node);
    if search.results.is_empty() {
        return "Empty".to_string();
    }
    let mut content = String::new();
    for (path, node_content) in search.results {
        let path = format!(" {path} ");
        content.push_str(&format!("{path:-^78}\n{node_content}\n"));
    }
    content
}

// Handles the depth-first-search to handle `step_content` with a filter
struct NodeFilterSearch<'a> {
    opts: &'a PrintStepsOptions,
    current_path: Vec<String>,
    results: Vec<(String, String)>,
}

impl<'a> NodeFilterSearch<'a> {
    fn new(opts: &'a PrintStepsOptions) -> Self {
        Self {
            opts,
            current_path: vec!["root".to_string()],
            results: vec![],
        }
    }

    fn search(&mut self, node: &dyn Node) {
        // If any child nodes match, then add this node to the results
        let mut child_match = false;
        node.visit_children(&mut |_, child| {
            child_match = child_match || self.opts.matches_node(node, child);
            Ok(())
        })
        .unwrap();
        if child_match {
            self.results
                .push((self.current_path.join(""), format!("{node:#?}")));
        } else {
            // Otherwise, continue recursing
            node.visit_children(&mut |field_name, child| {
                self.current_path.push(field_name.to_string());
                self.search(child);
                self.current_path.pop();
                Ok(())
            })
            .unwrap();
        }
    }
}
