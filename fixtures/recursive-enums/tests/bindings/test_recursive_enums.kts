/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.recursive_enums.*

// Tree (self-recursive)
val leaf = Tree.Leaf(5)
assert(sumTree(leaf) == 5)

val node = Tree.Node(Tree.Leaf(3), Tree.Leaf(4))
assert(sumTree(node) == 7)

val deep = Tree.Node(
    Tree.Node(Tree.Leaf(1), Tree.Leaf(2)),
    Tree.Leaf(10)
)
assert(sumTree(deep) == 13)

// Expr / BoolExpr (mutually recursive)
assert(evalExpr(Expr.Lit(42)) == 42)

val ifTrue = Expr.If(
    cond = BoolExpr.IsZero(Expr.Lit(0)),
    then = Expr.Lit(1),
    `else` = Expr.Lit(2)
)
assert(evalExpr(ifTrue) == 1)

val ifFalse = Expr.If(
    cond = BoolExpr.IsZero(Expr.Lit(5)),
    then = Expr.Lit(1),
    `else` = Expr.Lit(2)
)
assert(evalExpr(ifFalse) == 2)

assert(evalBool(BoolExpr.Not(BoolExpr.True)) == false)
assert(evalBool(BoolExpr.Not(BoolExpr.False)) == true)

// LinkedList (cycle through Optional: Cons.tail is LinkedList?)
assert(listSum(LinkedList.Nil) == 0)
assert(listSum(LinkedList.Cons(head = 5, tail = null)) == 5)
assert(listSum(LinkedList.Cons(head = 3, tail = LinkedList.Cons(head = 4, tail = null))) == 7)

// Trie (cycle through Map value: Branch.children is Map<String, Trie>)
assert(trieSum(Trie.Leaf(5)) == 5)
assert(trieSum(Trie.Branch(children = mapOf())) == 0)
assert(trieSum(Trie.Branch(children = mapOf("a" to Trie.Leaf(1), "b" to Trie.Leaf(2)))) == 3)
assert(trieSum(Trie.Branch(children = mapOf("x" to Trie.Branch(children = mapOf("y" to Trie.Leaf(7)))))) == 7)

// RoseTree (cycle through a record: RoseTree → RoseData → List<RoseTree>)
assert(sumRoseTree(RoseTree.Leaf(7)) == 7)

assert(sumRoseTree(RoseTree.Branch(RoseData(value = 10, children = listOf()))) == 10)

assert(sumRoseTree(RoseTree.Branch(
    RoseData(value = 1, children = listOf(RoseTree.Leaf(2), RoseTree.Leaf(3)))
)) == 6)

assert(sumRoseTree(RoseTree.Branch(RoseData(
    value = 1,
    children = listOf(
        RoseTree.Branch(RoseData(value = 2, children = listOf(RoseTree.Leaf(3)))),
        RoseTree.Leaf(4),
    )
))) == 10)

// EvalError (recursive error enum)
assert(maybeThrowError(false) == 42)
try {
    maybeThrowError(true)
    throw AssertionError("expected EvalError")
} catch (e: EvalException) {
    // recursive error enum compiled and is throwable
}
