# frozen_string_literal: true

require 'test/unit'
require 'todolist'

include Test::Unit::Assertions
include Todolist

todo = TodoList.new
entry = TodoEntry.new 'Write bindings for strings in records'

todo.add_item('Write ruby bindings')

assert_equal todo.get_last, 'Write ruby bindings'

todo.add_item('Write tests for bindings')

assert_equal todo.get_last, 'Write tests for bindings'

todo.add_entry(entry)

assert_equal todo.get_last, 'Write bindings for strings in records'
assert_equal todo.get_last_entry.text, 'Write bindings for strings in records'

todo.add_item("Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")
assert_equal todo.get_last, "Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣"

entry2 = TodoEntry.new("Test Ünicode hàndling in an entry can't believe I didn't test this at first 🤣")
todo.add_entry(entry2)
assert_equal todo.get_last_entry.text, "Test Ünicode hàndling in an entry can't believe I didn't test this at first 🤣"
