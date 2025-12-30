/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::any::{type_name, Any, TypeId};
use std::collections::{BTreeSet, HashSet};

use anyhow::{bail, Result};
use indexmap::{IndexMap, IndexSet};

/// Trait for IR types
///
/// This is typically implemented via `#[derive(Node)]`.  It implements some utility functions for
/// walking the IR tree.
pub trait Node: Any + std::fmt::Debug {
    /// struct/enum name
    fn type_name(&self) -> Option<&'static str> {
        None
    }

    fn as_any(&self) -> &dyn Any;

    fn to_box_any(self: Box<Self>) -> Box<dyn Any>;

    /// Call a visitor function for each direct child
    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn Node));

    /// Call a visitor function for each descendant
    fn try_visit_descendants(
        &self,
        visitor: &mut dyn FnMut(&dyn Node) -> Result<()>,
    ) -> Result<()> {
        // Store the result as we go through.  Once this is an Err value, short-circuit all
        // remaining work.
        let mut result = Ok(());
        self.visit_children(&mut |child| {
            if result.is_err() {
                return;
            }
            result = visitor(child);
            if result.is_err() {
                return;
            }
            result = child.try_visit_descendants(visitor);
        });
        result
    }

    /// Check if there are any descendants of this node with a particular type
    fn has_descendant_type<N: Node>(visited: &mut HashSet<TypeId>) -> bool
    where
        Self: Sized;

    /// Visit all descendants of a particular type
    ///
    /// `visit` panics if no descendant type matches `T`.
    fn visit<T: Node>(&self, mut visitor: impl FnMut(&T))
    where
        Self: Sized,
    {
        typecheck_visit::<Self, T>();
        self.try_visit_descendants(&mut |node| {
            if let Some(node) = node.as_any().downcast_ref::<T>() {
                visitor(node);
            }
            Ok(())
        })
        .unwrap(); // Unwrap is safe here, since the closure always returns `Ok(())`
    }

    /// Like `visit`, but with a fallible function.
    fn try_visit<T: Node>(&self, mut visitor: impl FnMut(&T) -> Result<()>) -> Result<()>
    where
        Self: Sized,
    {
        typecheck_visit::<Self, T>();
        self.try_visit_descendants(&mut |node| {
            if let Some(node) = node.as_any().downcast_ref::<T>() {
                visitor(node)?;
            }
            Ok(())
        })
    }

    /// Check if a predicate is true for any descendant node
    ///
    /// `has_descendant` panics if no descendant type matches `T`.
    fn has_descendant<T: Node>(&self, mut visitor: impl FnMut(&T) -> bool) -> bool
    where
        Self: Sized,
    {
        typecheck_visit::<Self, T>();
        self.try_visit_descendants(&mut |node| {
            if let Some(node) = node.as_any().downcast_ref::<T>() {
                if visitor(node) {
                    // Use `Err` to signal a match.
                    // This will short-circuit the rest of the work.
                    bail!("")
                }
            }
            Ok(())
        })
        .is_err()
    }

    /// Generate a string representation for this node
    fn repr(&self) -> String {
        format!("{self:#?}")
    }
}

fn typecheck_visit<N: Node, T: Node>() {
    if !N::has_descendant_type::<T>(&mut HashSet::default()) {
        panic!(
            "{} is not a descendant of {}",
            type_name::<T>(),
            type_name::<N>()
        );
    }
}

macro_rules! impl_leaf_nodes {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Node for $ty {
                fn as_any(&self) -> &dyn Any {
                    self
                }

                fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
                    self
                }

                fn visit_children(&self, _visitor: &mut dyn FnMut(&dyn Node)) { }

                fn has_descendant_type<N: Node>(_visited: &mut HashSet<TypeId>) -> bool {
                    false
                }
            }
        )*
    };
}

impl_leaf_nodes!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, String, bool,);

impl<T: Node> Node for Box<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn Node)) {
        (**self).visit_children(visitor);
    }

    fn has_descendant_type<N: Node>(visited: &mut HashSet<TypeId>) -> bool {
        if TypeId::of::<N>() == TypeId::of::<Self>() {
            return true;
        }
        if !visited.insert(TypeId::of::<Self>()) {
            return false;
        }
        T::has_descendant_type::<N>(visited)
    }
}

impl<T: Node> Node for Option<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn Node)) {
        if let Some(node) = self {
            visitor(node)
        }
    }

    fn has_descendant_type<N: Node>(visited: &mut HashSet<TypeId>) -> bool {
        if TypeId::of::<N>() == TypeId::of::<Self>() {
            return true;
        }
        if !visited.insert(TypeId::of::<Self>()) {
            return false;
        }
        T::has_descendant_type::<N>(visited)
    }
}

impl<T: Node> Node for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn Node)) {
        for node in self.iter() {
            visitor(node)
        }
    }

    fn has_descendant_type<N: Node>(visited: &mut HashSet<TypeId>) -> bool {
        if TypeId::of::<N>() == TypeId::of::<Self>() {
            return true;
        }
        if !visited.insert(TypeId::of::<Self>()) {
            return false;
        }
        T::has_descendant_type::<N>(visited)
    }
}

impl<T: Node> Node for BTreeSet<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn Node)) {
        for node in self.iter() {
            visitor(node)
        }
    }

    fn has_descendant_type<N: Node>(visited: &mut HashSet<TypeId>) -> bool {
        if TypeId::of::<N>() == TypeId::of::<Self>() {
            return true;
        }
        if !visited.insert(TypeId::of::<Self>()) {
            return false;
        }
        T::has_descendant_type::<N>(visited)
    }
}

impl<T: Node> Node for IndexSet<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn Node)) {
        for node in self.iter() {
            visitor(node)
        }
    }

    fn has_descendant_type<N: Node>(visited: &mut HashSet<TypeId>) -> bool {
        if TypeId::of::<N>() == TypeId::of::<Self>() {
            return true;
        }
        if !visited.insert(TypeId::of::<Self>()) {
            return false;
        }
        T::has_descendant_type::<N>(visited)
    }
}

impl<K: Node, V: Node> Node for IndexMap<K, V> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn Node)) {
        for (key, value) in self.iter() {
            visitor(key);
            visitor(value);
        }
    }

    fn has_descendant_type<N: Node>(visited: &mut HashSet<TypeId>) -> bool {
        if TypeId::of::<N>() == TypeId::of::<Self>() {
            return true;
        }
        if !visited.insert(TypeId::of::<Self>()) {
            return false;
        }
        K::has_descendant_type::<N>(visited) || V::has_descendant_type::<N>(visited)
    }
}
