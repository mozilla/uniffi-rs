/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::BTreeSet, hash::Hash};

use anyhow::Result;
use indexmap::{IndexMap, IndexSet};

/// Trait for converting one node type to another
///
/// This can be auto-implemented using `#[derive(MapNode)]` for nodes that:
///
/// - Have not added any fields (removed is okay)
/// - All fields implement `MapNode` for the previous node type.
///
/// The `context` argument exists to support manual implementations.  It allows ancestor nodes to
/// pass down data for child nodes to use.  For example the namespace name or current `self_type`.
pub trait MapNode<Output, Context> {
    fn map_node(self, context: &Context) -> Result<Output>
    where
        Self: Sized;
}

macro_rules! simple_nodes {
    ($($ty:ty),* $(,)?) => {
        $(
            impl<C> MapNode<$ty, C> for $ty {
                fn map_node(self, _context: &C) -> Result<Self> {
                    Ok(self)
                }
            }
        )*
    };
}

simple_nodes!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, String, bool,);

impl<Input, Output, Context> MapNode<Box<Output>, Context> for Box<Input>
where
    Input: MapNode<Output, Context>,
{
    fn map_node(self, context: &Context) -> Result<Box<Output>> {
        Input::map_node(*self, context).map(Box::new)
    }
}

impl<Input, Output, Context> MapNode<Option<Output>, Context> for Option<Input>
where
    Input: MapNode<Output, Context>,
{
    fn map_node(self, context: &Context) -> Result<Option<Output>> {
        self.map(|input| Input::map_node(input, context))
            .transpose()
    }
}

impl<Input, Output, Context> MapNode<Vec<Output>, Context> for Vec<Input>
where
    Input: MapNode<Output, Context>,
{
    fn map_node(self, context: &Context) -> Result<Vec<Output>> {
        self.into_iter()
            .map(|input| Input::map_node(input, context))
            .collect()
    }
}

impl<Input, Output, Context> MapNode<BTreeSet<Output>, Context> for BTreeSet<Input>
where
    Input: MapNode<Output, Context>,
    Output: Ord,
{
    fn map_node(self, context: &Context) -> Result<BTreeSet<Output>> {
        self.into_iter()
            .map(|input| Input::map_node(input, context))
            .collect()
    }
}

impl<Input, Output, Context> MapNode<IndexSet<Output>, Context> for IndexSet<Input>
where
    Input: MapNode<Output, Context>,
    Output: Hash + Eq,
{
    fn map_node(self, context: &Context) -> Result<IndexSet<Output>> {
        self.into_iter()
            .map(|input| Input::map_node(input, context))
            .collect()
    }
}

impl<InputKey, InputValue, OutputKey, OutputValue, Context>
    MapNode<IndexMap<OutputKey, OutputValue>, Context> for IndexMap<InputKey, InputValue>
where
    InputKey: MapNode<OutputKey, Context>,
    InputValue: MapNode<OutputValue, Context>,
    OutputKey: Hash + Eq,
{
    fn map_node(self, context: &Context) -> Result<IndexMap<OutputKey, OutputValue>> {
        self.into_iter()
            .map(|(key, value)| {
                Ok((
                    InputKey::map_node(key, context)?,
                    InputValue::map_node(value, context)?,
                ))
            })
            .collect()
    }
}
