import uniffi.todolist.*

val todo = TodoList()
todo.addItem("Write strings support")

assert(todo.getLast() == "Write strings support")

todo.addItem("Write tests for strings support")

assert(todo.getLast() == "Write tests for strings support")

val entry = TodoEntry("Write bindings for strings as record members")
todo.addEntry(entry)

assert(todo.getLast() == "Write bindings for strings as record members")
