/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Test recursive types and boxed fields

// Self-recursive tree
#[derive(uniffi::Enum)]
pub enum Tree {
    Leaf(i32),
    Node { left: Box<Tree>, right: Box<Tree> },
}

// Mutually recursive pair
#[derive(uniffi::Enum)]
pub enum Expr {
    Lit(i32),
    If {
        cond: Box<BoolExpr>,
        then_: Box<Expr>,
        else_: Box<Expr>,
    },
}

#[derive(uniffi::Enum)]
pub enum BoolExpr {
    True_,
    False_,
    Not(Box<BoolExpr>),
    IsZero(Box<Expr>),
}

// Cycle through Optional: LinkedList → Option<Box<LinkedList>>
#[derive(uniffi::Enum)]
pub enum LinkedList {
    Nil,
    Cons {
        head: i32,
        tail: Option<Box<LinkedList>>,
    },
}

#[uniffi::export]
pub fn list_sum(list: LinkedList) -> i32 {
    match list {
        LinkedList::Nil => 0,
        LinkedList::Cons { head, tail } => head + tail.map(|t| list_sum(*t)).unwrap_or(0),
    }
}

// Cycle through Map value: Trie → HashMap<String, Box<Trie>>
#[derive(uniffi::Enum)]
pub enum Trie {
    Leaf(i32),
    Branch {
        children: std::collections::HashMap<String, Box<Trie>>,
    },
}

#[uniffi::export]
pub fn trie_sum(trie: Trie) -> i32 {
    match trie {
        Trie::Leaf(v) => v,
        Trie::Branch { children } => children.into_values().map(|t| trie_sum(*t)).sum(),
    }
}

// Cycle through a record: RoseTree → RoseData → Vec<RoseTree>
#[derive(uniffi::Enum)]
pub enum RoseTree {
    Leaf(i32),
    Branch(RoseData),
}

#[derive(uniffi::Record)]
pub struct RoseData {
    pub value: i32,
    pub children: Vec<RoseTree>,
}

#[uniffi::export]
pub fn sum_rose_tree(tree: RoseTree) -> i32 {
    match tree {
        RoseTree::Leaf(v) => v,
        RoseTree::Branch(data) => {
            data.value + data.children.into_iter().map(sum_rose_tree).sum::<i32>()
        }
    }
}

#[uniffi::export]
pub fn sum_tree(tree: Tree) -> i32 {
    match tree {
        Tree::Leaf(v) => v,
        Tree::Node { left, right } => sum_tree(*left) + sum_tree(*right),
    }
}

#[uniffi::export]
pub fn eval_expr(expr: Expr) -> i32 {
    match expr {
        Expr::Lit(v) => v,
        Expr::If { cond, then_, else_ } => {
            if eval_bool(*cond) {
                eval_expr(*then_)
            } else {
                eval_expr(*else_)
            }
        }
    }
}

#[uniffi::export]
pub fn eval_bool(expr: BoolExpr) -> bool {
    match expr {
        BoolExpr::True_ => true,
        BoolExpr::False_ => false,
        BoolExpr::Not(inner) => !eval_bool(*inner),
        BoolExpr::IsZero(inner) => eval_expr(*inner) == 0,
    }
}

// Recursive error enum: Nested(Box<EvalError>) forms a self-cycle.
// Verifies that #[derive(uniffi::Error)] enums are detected as recursive
// and generate correct indirect/forward-ref bindings.
#[derive(Debug, uniffi::Error)]
pub enum EvalError {
    Overflow,
    Nested { inner: Box<EvalError> },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::Overflow => write!(f, "overflow"),
            EvalError::Nested { .. } => write!(f, "nested error"),
        }
    }
}

impl std::error::Error for EvalError {}

#[uniffi::export]
pub fn maybe_throw_error(should_throw: bool) -> Result<i32, EvalError> {
    if should_throw {
        Err(EvalError::Nested {
            inner: Box::new(EvalError::Overflow),
        })
    } else {
        Ok(42)
    }
}
