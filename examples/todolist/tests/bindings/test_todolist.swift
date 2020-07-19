import todolist

let todo = TodoList()
todo.add_item(todo: "Write swift bindings")
assert( todo.get_last() == "Write swift bindings")

todo.add_item(todo: "Write tests for bindings")
assert(todo.get_last() == "Write tests for bindings")
