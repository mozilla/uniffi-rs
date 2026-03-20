/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// Tree (self-recursive)
let leaf = Tree.leaf(5)
assert(sumTree(tree: leaf) == 5)

let node = Tree.node(left: Tree.leaf(3), right: Tree.leaf(4))
assert(sumTree(tree: node) == 7)

let deep = Tree.node(
    left: Tree.node(left: Tree.leaf(1), right: Tree.leaf(2)),
    right: Tree.leaf(10)
)
assert(sumTree(tree: deep) == 13)

// Expr / BoolExpr (mutually recursive)
assert(evalExpr(expr: Expr.lit(42)) == 42)

// BoolExpr.isZero is true when expr evaluates to zero — takes the `then` branch
let ifTrue = Expr.`if`(
    cond: BoolExpr.isZero(Expr.lit(0)),
    then: Expr.lit(1),
    else: Expr.lit(2)
)
assert(evalExpr(expr: ifTrue) == 1)

// BoolExpr.isZero is false when expr is nonzero — takes the `else` branch
let ifFalse = Expr.`if`(
    cond: BoolExpr.isZero(Expr.lit(5)),
    then: Expr.lit(1),
    else: Expr.lit(2)
)
assert(evalExpr(expr: ifFalse) == 2)

// BoolExpr.not wraps another BoolExpr
assert(evalBool(expr: BoolExpr.not(BoolExpr.`true`)) == false)
assert(evalBool(expr: BoolExpr.not(BoolExpr.`false`)) == true)

// LinkedList (cycle through Optional: Cons.tail is LinkedList?)
assert(listSum(list: .`nil`) == 0)
assert(listSum(list: .cons(head: 5, tail: nil)) == 5)
assert(listSum(list: .cons(head: 3, tail: .cons(head: 4, tail: nil))) == 7)

// Trie (cycle through Map value: Branch.children is [String: Trie])
assert(trieSum(trie: .leaf(5)) == 5)
assert(trieSum(trie: .branch(children: [:])) == 0)
assert(trieSum(trie: .branch(children: ["a": .leaf(1), "b": .leaf(2)])) == 3)
assert(trieSum(trie: .branch(children: ["x": .branch(children: ["y": .leaf(7)])])) == 7)

// RoseTree (cycle through a record: RoseTree → RoseData → [RoseTree])
assert(sumRoseTree(tree: RoseTree.leaf(7)) == 7)

assert(sumRoseTree(tree: RoseTree.branch(RoseData(value: 10, children: []))) == 10)

assert(sumRoseTree(tree: RoseTree.branch(
    RoseData(value: 1, children: [RoseTree.leaf(2), RoseTree.leaf(3)])
)) == 6)

assert(sumRoseTree(tree: RoseTree.branch(RoseData(
    value: 1,
    children: [
        RoseTree.branch(RoseData(value: 2, children: [RoseTree.leaf(3)])),
        RoseTree.leaf(4),
    ]
))) == 10)

// EvalError (recursive error enum)
assert((try? maybeThrowError(shouldThrow: false)) == 42)
do {
    _ = try maybeThrowError(shouldThrow: true)
    assert(false, "expected EvalError to be thrown")
} catch {
    // recursive error enum compiled and is throwable
}
