# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestRecursiveTypes < Test::Unit::TestCase
  include UniffiBindgenTests

  # --- Self-recursive tree ---

  def test_tree_leaf
    assert_equal 42, UniffiBindgenTests.sum_tree(Tree::LEAF.new(42))
  end

  def test_tree_node
    tree = Tree::NODE.new(
      left: Tree::LEAF.new(1),
      right: Tree::LEAF.new(2)
    )

    assert_equal 3, UniffiBindgenTests.sum_tree(tree)
  end

  def test_tree_nested
    tree = Tree::NODE.new(
      left: Tree::NODE.new(
        left: Tree::LEAF.new(1),
        right: Tree::LEAF.new(2)
      ),
      right: Tree::LEAF.new(3)
    )

    assert_equal 6, UniffiBindgenTests.sum_tree(tree)
  end

  # --- Mutally recursive Expr / BoolExpr ---

  def test_expr_lit
    assert_equal 42, UniffiBindgenTests.eval_expr(Expr::LIT.new(42))
  end

  def test_expr_if_true
    expr = Expr::IF.new(
      cond: BoolExpr::TRUE.new,
      _then: Expr::LIT.new(1),
      _else: Expr::LIT.new(0)
    )

    assert_equal 1, UniffiBindgenTests.eval_expr(expr)
  end

  def test_expr_if_false
    expr = Expr::IF.new(
      cond: BoolExpr::FALSE.new,
      _then: Expr::LIT.new(1),
      _else: Expr::LIT.new(0)
    )

    assert_equal 0, UniffiBindgenTests.eval_expr(expr)
  end

  def test_bool_not
    assert UniffiBindgenTests.eval_bool(BoolExpr::NOT.new(BoolExpr::FALSE.new))
    assert !UniffiBindgenTests.eval_bool(BoolExpr::NOT.new(BoolExpr::TRUE.new))
  end

  def test_bool_is_zero
    assert UniffiBindgenTests.eval_bool(BoolExpr::IS_ZERO.new(Expr::LIT.new(0)))
    assert !UniffiBindgenTests.eval_bool(BoolExpr::IS_ZERO.new(Expr::LIT.new(5)))
  end

  # --- LinkedList ---

  def test_linked_list_nil
    assert_equal 0, UniffiBindgenTests.list_sum(LinkedList::NIL.new)
  end

  def test_linked_list_single
    list = LinkedList::CONS.new head: 5, tail: nil

    assert_equal 5, UniffiBindgenTests.list_sum(list)
  end

  def test_linked_list_multiple
    list = LinkedList::CONS.new(
      head: 1,
      tail: LinkedList::CONS.new(
        head: 2,
        tail: LinkedList::CONS.new(head: 3, tail: nil)
      )
    )

    assert_equal 6, UniffiBindgenTests.list_sum(list)
  end

  # -- Trie --

  def test_trie_leaf
    assert_equal 10, UniffiBindgenTests.trie_sum(Trie::LEAF.new(10))
  end

  def test_trie_branch
    trie = Trie::BRANCH.new(children: {
                              'a' => Trie::LEAF.new(1),
                              'b' => Trie::LEAF.new(2)
                            })

    assert_equal 3, UniffiBindgenTests.trie_sum(trie)
  end

  # -- RoseTree --

  def test_rose_tree_leaf
    assert_equal 5, UniffiBindgenTests.sum_rose_tree(RoseTree::LEAF.new(5))
  end

  def test_rose_tree_branch
    tree = RoseTree::BRANCH.new(
      RoseData.new(
        value: 1,
        children: [
          RoseTree::LEAF.new(2),
          RoseTree::LEAF.new(3)
        ]
      )
    )

    assert_equal 6, UniffiBindgenTests.sum_rose_tree(tree)
  end

  # --- Recursive EvalError ---

  def test_maybe_throw_no_error
    assert_equal 42, UniffiBindgenTests.maybe_throw_error(false)
  end

  def test_maybe_throw_error
    assert_raises(EvalError::Nested) { UniffiBindgenTests.maybe_throw_error(true) }
  end
end
