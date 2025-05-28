/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{any::Any, fmt, hash::Hash};

use anyhow::{anyhow, Result};
use indexmap::{IndexMap, IndexSet};

use super::Value;

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
    ///
    /// This is used to implement the type name filter in the CLI
    fn type_name(&self) -> Option<&'static str> {
        None
    }

    /// Create a value from this node's data
    ///
    /// Logically, this consumes `self`.  However, it inputs `&mut self` because it works better
    /// with dyn traits.  It leaves behind an empty node.
    fn take_into_value(&mut self) -> Value;

    /// Convert a value into this node.
    fn try_from_value(value: Value) -> Result<Self, FromValueError>
    where
        Self: Sized;

    /// Convert `&Node` into `&dyn Any`.
    ///
    /// This is used to implement `visit_children`.
    fn as_any(&self) -> &dyn Any;

    /// Convert `&Node` into `&mut dyn Any`.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn to_box_any(self: Box<Self>) -> Box<dyn Any>;

    fn visit<T: Node>(&self, mut visitor: impl FnMut(&T))
    where
        Self: Sized,
    {
        // unwrap() should never panic, since the visitor doesn't return an error
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
        // unwrap() should never panic, since the visitor doesn't return an error
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

    /// Take the current value from `self` leaving behind an empty value
    fn take(&mut self) -> Self
    where
        Self: Default,
    {
        std::mem::take(self)
    }

    /// Convert from one node to another
    fn try_from_node(mut other: impl Node) -> Result<Self>
    where
        Self: Sized,
    {
        Self::try_from_value(other.take_into_value()).map_err(|mut e| {
            e.path.reverse();
            let path = e.path.join("");
            anyhow!("Node conversion error: {} (path: <root>{path})", e.message)
        })
    }
}

impl dyn Node {
    pub(crate) fn try_visit_descendents_recurse<T: Node>(
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

    pub(crate) fn try_visit_descendents_recurse_mut<T: Node>(
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

/// Error struct for `Node::try_from_value`
pub struct FromValueError {
    /// Field path to the node that couldn't be converted.
    ///
    /// To make the generated code simpler, this is in reverse order, the first items in the vec
    /// are the deepest items in the node tree.
    path: Vec<String>,
    message: String,
}

impl FromValueError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            path: vec![],
        }
    }

    pub fn add_field_to_path(mut self, field: impl Into<String>) -> Self {
        self.path.push(field.into());
        self
    }

    pub fn into_anyhow(mut self) -> anyhow::Error {
        self.path.reverse();
        let path = self.path.join("");
        anyhow!(
            "Node conversion error: {} (path: <root>{path})",
            self.message
        )
    }
}

macro_rules! simple_nodes {
    ($($value_variant:ident($ty:ty)),* $(,)?) => {
        $(
            impl Node for $ty {
                fn as_any(&self) -> &dyn Any {
                    self
                }

                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }

                fn to_box_any(self: Box<Self>) -> Box<dyn Any>{
                    self
                }

                fn take_into_value(&mut self) -> Value {
                    Value::$value_variant(self.take())
                }

                fn try_from_value(value: Value) -> Result<Self, FromValueError> {
                    match value {
                        Value::$value_variant(v) => Ok(v),
                        v => Err(FromValueError::new(format!(
                            "Node type error (expected {}, actual {v:?})",
                            std::stringify!($value_variant)
                        ))),
                    }
                }
            }
        )*
    };
}

simple_nodes!(
    UInt8(u8),
    Int8(i8),
    UInt16(u16),
    Int16(i16),
    UInt32(u32),
    Int32(i32),
    UInt64(u64),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    Bool(bool),
);

impl<T: Node> Node for Box<T> {
    fn visit_children(&self, visitor: &mut dyn FnMut(&str, &dyn Node) -> Result<()>) -> Result<()> {
        visitor("", &**self)
    }

    fn visit_children_mut(
        &mut self,
        visitor: &mut dyn FnMut(&str, &mut dyn Node) -> Result<()>,
    ) -> Result<()> {
        visitor("", &mut **self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn take_into_value(&mut self) -> Value {
        (**self).take_into_value()
    }

    fn try_from_value(value: Value) -> Result<Self, FromValueError> {
        T::try_from_value(value).map(Box::new)
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

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn take_into_value(&mut self) -> Value {
        Value::Option(
            self.take()
                .map(|mut node| node.take_into_value())
                .map(Box::new),
        )
    }

    fn try_from_value(value: Value) -> Result<Self, FromValueError> {
        match value {
            Value::Option(v) => v.map(|v| v.try_into_node()).transpose(),
            v => Err(FromValueError::new(format!(
                "Node type error (expected Option, actual {v:?})"
            ))),
        }
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

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn take_into_value(&mut self) -> Value {
        Value::Vec(
            self.take()
                .into_iter()
                .map(|mut node| node.take_into_value())
                .collect(),
        )
    }

    fn try_from_value(value: Value) -> Result<Self, FromValueError> {
        match value {
            Value::Vec(values) => values
                .into_iter()
                .enumerate()
                .map(|(i, v)| {
                    v.try_into_node()
                        .map_err(|e| e.add_field_to_path(format!("[{i}]")))
                })
                .collect(),
            v => Err(FromValueError::new(format!(
                "Node type error (expected Vec, actual {v:?})",
            ))),
        }
    }
}

impl<T> Node for IndexSet<T>
where
    T: Node + Hash + Eq,
{
    fn visit_children(&self, visitor: &mut dyn FnMut(&str, &dyn Node) -> Result<()>) -> Result<()> {
        for v in self.iter() {
            visitor(&format!("{{{v:?}}}"), v)?;
        }
        Ok(())
    }

    fn visit_children_mut(
        &mut self,
        visitor: &mut dyn FnMut(&str, &mut dyn Node) -> Result<()>,
    ) -> Result<()> {
        // We can't directly mutate IndexSet items, since that would change their hash value.
        // Instead, use deconstruct the set using into_iter() and collect everything at the end.
        *self = Node::take(self)
            .into_iter()
            .map(|mut v| {
                visitor(&format!("{{{v:?}}}"), &mut v)?;
                Ok(v)
            })
            .collect::<Result<IndexSet<_>>>()?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn take_into_value(&mut self) -> Value {
        Value::Set(
            Node::take(self)
                .into_iter()
                .map(|mut node| node.take_into_value())
                .collect(),
        )
    }

    fn try_from_value(value: Value) -> Result<Self, FromValueError> {
        match value {
            Value::Set(values) => values
                .into_iter()
                .map(|k| {
                    let key_name = format!("{k:?}");
                    k.try_into_node()
                        .map_err(|e| e.add_field_to_path(format!("{{{key_name:?}}}")))
                })
                .collect(),
            v => Err(FromValueError::new(format!(
                "Node type error (expected Map, actual {v:?})",
            ))),
        }
    }
}

impl<K, V> Node for IndexMap<K, V>
where
    K: Node + Hash + Eq,
    V: Node,
{
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
        // We can't directly mutate Map key, since that would change their hash value.
        // Instead, use deconstruct the set using into_iter() and collect everything at the end.
        *self = self
            .take()
            .into_iter()
            .map(|(mut k, mut v)| {
                let key_name = format!("{k:?}");
                visitor(&format!("{{{key_name:?}}}"), &mut k)?;
                visitor(&format!("[{key_name:?}]"), &mut v)?;
                Ok((k, v))
            })
            .collect::<Result<IndexMap<_, _>>>()?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn take_into_value(&mut self) -> Value {
        Value::Map(
            self.take()
                .into_iter()
                .map(|(mut k, mut v)| (k.take_into_value(), v.take_into_value()))
                .collect(),
        )
    }

    fn try_from_value(value: Value) -> Result<Self, FromValueError> {
        match value {
            Value::Map(values) => values
                .into_iter()
                .map(|(k, v)| {
                    let v = v
                        .try_into_node()
                        .map_err(|e| e.add_field_to_path(format!("[{k:?}]")))?;
                    let k = k
                        .try_into_node()
                        .map_err(|e| e.add_field_to_path("[<key>]".to_string()))?;
                    Ok((k, v))
                })
                .collect(),
            v => Err(FromValueError::new(format!(
                "Node type error (expected Map, actual {v:?})",
            ))),
        }
    }
}
