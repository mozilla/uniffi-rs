# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from recursive_enums import *

# --- Tree (self-recursive) ---

# Leaf
assert sum_tree(Tree.LEAF(5)) == 5

# Node with two leaves
tree = Tree.NODE(left=Tree.LEAF(3), right=Tree.LEAF(4))
assert sum_tree(tree) == 7

# Deeper tree
deep = Tree.NODE(
    left=Tree.NODE(left=Tree.LEAF(1), right=Tree.LEAF(2)),
    right=Tree.LEAF(10),
)
assert sum_tree(deep) == 13

# --- Expr / BoolExpr (mutually recursive) ---

# Simple literal
assert eval_expr(Expr.LIT(42)) == 42

# IsZero true branch
# Note: then_ field becomes `then`, else_ becomes `_else` (else is a Python keyword)
expr = Expr.IF(
    cond=BoolExpr.IS_ZERO(Expr.LIT(0)),
    then=Expr.LIT(1),
    _else=Expr.LIT(2),
)
assert eval_expr(expr) == 1

# IsZero false branch
expr = Expr.IF(
    cond=BoolExpr.IS_ZERO(Expr.LIT(5)),
    then=Expr.LIT(1),
    _else=Expr.LIT(2),
)
assert eval_expr(expr) == 2

# Not
assert eval_bool(BoolExpr.NOT(BoolExpr.TRUE())) == False
assert eval_bool(BoolExpr.NOT(BoolExpr.FALSE())) == True

# --- LinkedList (cycle through Optional: Cons.tail is Option<LinkedList>) ---

assert list_sum(LinkedList.NIL()) == 0
assert list_sum(LinkedList.CONS(head=5, tail=None)) == 5
assert list_sum(LinkedList.CONS(head=3, tail=LinkedList.CONS(head=4, tail=None))) == 7

# --- Trie (cycle through Map value: Branch.children is HashMap<String, Trie>) ---

assert trie_sum(Trie.LEAF(5)) == 5
assert trie_sum(Trie.BRANCH(children={})) == 0
assert trie_sum(Trie.BRANCH(children={"a": Trie.LEAF(1), "b": Trie.LEAF(2)})) == 3
assert trie_sum(Trie.BRANCH(children={"x": Trie.BRANCH(children={"y": Trie.LEAF(7)})})) == 7

# --- RoseTree (cycle through a record: RoseTree → RoseData → Vec<RoseTree>) ---

assert sum_rose_tree(RoseTree.LEAF(7)) == 7

assert sum_rose_tree(RoseTree.BRANCH(RoseData(value=10, children=[]))) == 10

assert sum_rose_tree(
    RoseTree.BRANCH(RoseData(value=1, children=[RoseTree.LEAF(2), RoseTree.LEAF(3)]))
) == 6

assert sum_rose_tree(
    RoseTree.BRANCH(RoseData(
        value=1,
        children=[
            RoseTree.BRANCH(RoseData(value=2, children=[RoseTree.LEAF(3)])),
            RoseTree.LEAF(4),
        ],
    ))
) == 10

# EvalError (recursive error enum: Nested wraps another EvalError)
assert maybe_throw_error(False) == 42

try:
    maybe_throw_error(True)
    assert False, "expected EvalError to be raised"
except EvalError:
    pass  # recursive error enum compiles and is throwable
