/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{any::Any, fmt};

use anyhow::Result;
use indexmap::IndexMap;

/// Node trait, this is implemented on all nodes in the Ir (structs, enums and their fields)
///
/// Note: All node types must implement `Clone`, `Default`, and `PartialEq`, and the derive macro ensures that.
/// However, these are not bounds on the node trait, since they're not `dyn-compatible`.
pub trait Node: fmt::Debug + Any {
    /// Call a visitor function for all child nodes
    ///
    /// Calls the visitor with a path string that represents the field name, vec index, etc. along
    /// with the child node.
    ///
    /// If the visitor returns an error, then the error is returned from `visit_children` without
    /// any more visits.
    fn visit_children(
        &self,
        _visitor: &mut dyn FnMut(&str, &dyn Node) -> Result<()>,
    ) -> Result<()> {
        Ok(())
    }

    /// Like visit_children, but use &mut.
    ///
    /// Note: this will not visit `IndexMap` keys, since they can't be mutated.
    fn visit_children_mut(
        &mut self,
        _visitor: &mut dyn FnMut(&str, &mut dyn Node) -> Result<()>,
    ) -> Result<()> {
        Ok(())
    }

    /// Type name for structs / enums
    fn type_name(&self) -> Option<&'static str> {
        None
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Empty value, used when adding new fields to IR types
    fn empty() -> Self
    where
        Self: Sized;

    fn visit<T: Node>(&self, mut visitor: impl FnMut(&T))
    where
        Self: Sized,
    {
        // Note: unwrap() should never panic, since the visitor doesn't return an error
        (self as &dyn Node)
            .try_visit_descendents_recurse(&mut |node| {
                visitor(node);
                Ok(())
            })
            .unwrap()
    }

    fn visit_mut<T: Node>(&mut self, mut visitor: impl FnMut(&mut T))
    where
        Self: Sized,
    {
        // Note: unwrap() should never panic, since the visitor doesn't return an error
        (self as &mut dyn Node)
            .try_visit_descendents_recurse_mut(&mut |node| {
                visitor(node);
                Ok(())
            })
            .unwrap()
    }

    fn try_visit<T: Node>(&self, mut visitor: impl FnMut(&T) -> Result<()>) -> Result<()>
    where
        Self: Sized,
    {
        (self as &dyn Node).try_visit_descendents_recurse(&mut visitor)
    }

    fn try_visit_mut<T: Node>(
        &mut self,
        mut visitor: impl FnMut(&mut T) -> Result<()>,
    ) -> Result<()>
    where
        Self: Sized,
    {
        (self as &mut dyn Node).try_visit_descendents_recurse_mut(&mut visitor)
    }

    /// Take the current value from `self` leaving behind a default value
    ///
    /// If fields are added and removed in a pass, use this to take the value from the removed
    /// field and use it for the added field.
    fn take(&mut self) -> Self
    where
        Self: Default,
    {
        std::mem::take(self)
    }
}

impl dyn Node {
    fn try_visit_descendents_recurse<T: Node>(
        &self,
        visitor: &mut dyn FnMut(&T) -> Result<()>,
    ) -> Result<()> {
        if let Some(node) = self.as_any().downcast_ref::<T>() {
            visitor(node)?;
        }
        self.visit_children(&mut |_, child| {
            child.try_visit_descendents_recurse(visitor)?;
            Ok(())
        })
    }

    fn try_visit_descendents_recurse_mut<T: Node>(
        &mut self,
        visitor: &mut dyn FnMut(&mut T) -> Result<()>,
    ) -> Result<()> {
        if let Some(node) = self.as_any_mut().downcast_mut::<T>() {
            visitor(node)?;
        }
        self.visit_children_mut(&mut |_, child| {
            child.try_visit_descendents_recurse_mut(visitor)?;
            Ok(())
        })
    }
}

/// Convert a node into the corresponding one in the next Ir.
///
/// This works exactly like the normal TryInto trait, but with different blanket impls.
/// We create blanket impls for `Vec<T: Node>`, rather than for `T` -> `T`
pub trait IntoNode<T> {
    fn into_node(self) -> Result<T>;
}

/// Convert a node from the corresponding one in the previous Ir.
pub trait FromNode<T>: Sized {
    fn from_node(value: T) -> Result<Self>;
}

impl<PrevNode, Node> IntoNode<Node> for PrevNode
where
    Node: FromNode<PrevNode>,
{
    fn into_node(self) -> Result<Node> {
        Node::from_node(self)
    }
}

macro_rules! simple_nodes {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Node for $ty {
                fn as_any(&self) -> &dyn Any {
                    self
                }

                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }

                fn empty() -> Self {
                    Self::default()
                }
            }

            impl FromNode<$ty> for $ty {
                fn from_node(value: $ty) -> Result<Self> {
                    Ok(value)
                }
            }
        )*
    };
}

simple_nodes!(String, bool, u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

impl<T: Node> Node for Box<T> {
    fn visit_children(&self, visitor: &mut dyn FnMut(&str, &dyn Node) -> Result<()>) -> Result<()> {
        (**self).visit_children(visitor)
    }

    fn visit_children_mut(
        &mut self,
        visitor: &mut dyn FnMut(&str, &mut dyn Node) -> Result<()>,
    ) -> Result<()> {
        (**self).visit_children_mut(visitor)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn empty() -> Self {
        Box::new(T::empty())
    }
}

impl<Node, PrevNode> FromNode<Box<PrevNode>> for Box<Node>
where
    Node: FromNode<PrevNode>,
{
    fn from_node(value: Box<PrevNode>) -> Result<Self> {
        (*value).into_node().map(Box::new)
    }
}

impl<T: Node> Node for Option<T> {
    fn visit_children(&self, visitor: &mut dyn FnMut(&str, &dyn Node) -> Result<()>) -> Result<()> {
        if let Some(node) = self {
            visitor("", node)?;
        }
        Ok(())
    }

    fn visit_children_mut(
        &mut self,
        visitor: &mut dyn FnMut(&str, &mut dyn Node) -> Result<()>,
    ) -> Result<()> {
        if let Some(node) = self {
            visitor("", node)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn empty() -> Self {
        None
    }
}

impl<Node, PrevNode> FromNode<Option<PrevNode>> for Option<Node>
where
    Node: FromNode<PrevNode>,
{
    fn from_node(value: Option<PrevNode>) -> Result<Self> {
        value.map(IntoNode::into_node).transpose()
    }
}

impl<T: Node> Node for Vec<T> {
    fn visit_children(&self, visitor: &mut dyn FnMut(&str, &dyn Node) -> Result<()>) -> Result<()> {
        for (i, child) in self.iter().enumerate() {
            visitor(&format!(".{i}"), child)?;
        }
        Ok(())
    }

    fn visit_children_mut(
        &mut self,
        visitor: &mut dyn FnMut(&str, &mut dyn Node) -> Result<()>,
    ) -> Result<()> {
        for (i, child) in self.iter_mut().enumerate() {
            visitor(&format!(".{i}"), child)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn empty() -> Self {
        Vec::default()
    }
}

impl<Node, PrevNode> FromNode<Vec<PrevNode>> for Vec<Node>
where
    Node: FromNode<PrevNode>,
{
    fn from_node(value: Vec<PrevNode>) -> Result<Self> {
        value.into_iter().map(IntoNode::into_node).collect()
    }
}

impl<K: Node, V: Node> Node for IndexMap<K, V> {
    fn visit_children(&self, visitor: &mut dyn FnMut(&str, &dyn Node) -> Result<()>) -> Result<()> {
        for (k, v) in self.iter() {
            visitor(&format!("[{k:?}]"), v)?;
            visitor(&format!(".key[{k:?}]"), k)?;
        }
        Ok(())
    }

    fn visit_children_mut(
        &mut self,
        visitor: &mut dyn FnMut(&str, &mut dyn Node) -> Result<()>,
    ) -> Result<()> {
        for (k, v) in self.iter_mut() {
            visitor(&format!("[{k:?}]"), v)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn empty() -> Self {
        IndexMap::default()
    }
}

// IndexMap is the preferred map type, since it keeps the key the order stable.
impl<KeyNode, PrevKeyNode, Node, PrevNode> FromNode<IndexMap<PrevKeyNode, PrevNode>>
    for IndexMap<KeyNode, Node>
where
    KeyNode: FromNode<PrevKeyNode>,
    Node: FromNode<PrevNode>,
    KeyNode: Eq + std::hash::Hash,
{
    fn from_node(value: IndexMap<PrevKeyNode, PrevNode>) -> Result<Self> {
        value
            .into_iter()
            .map(|(k, v)| Ok((k.into_node()?, v.into_node()?)))
            .collect()
    }
}
