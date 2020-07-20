from todolist import *

todo = TodoList()

entry = TodoEntry("Write bindings for strings in records")

todo.add_item("Write python bindings")

assert(todo.get_last() == "Write python bindings")

todo.add_item("Write tests for bindings")

assert(todo.get_last() == "Write tests for bindings")

todo.add_entry(entry)

assert(todo.get_last() == "Write bindings for strings in records")
