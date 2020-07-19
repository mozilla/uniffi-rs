from todolist import *

todo = TodoList()

todo.add_item("Write python bindings")

assert(todo.get_last() == "Write python bindings")

todo.add_item("Write tests for bindings")

assert(todo.get_last() == "Write tests for bindings")
