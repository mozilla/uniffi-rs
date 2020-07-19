import uniffi.todolist.*

val todo = TodoList()
todo.addItem("Write strings support")

assert(todo.getLast() == "Write strings support")

todo.addItem("Write tests for strings support")

assert(todo.getLast() == "Write tests for strings support")

val entry = TodoEntry("Write bindings for strings as record members")

todo.addEntry(entry)
assert(todo.getLast() == "Write bindings for strings as record members")
assert(todo.getLastEntry().text == "Write bindings for strings as record members")

todo.addItem("Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")
assert(todo.getLast() == "Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")

val entry2 = TodoEntry("Test Ünicode hàndling in an entry can't believe I didn't test this at first 🤣")
todo.addEntry(entry2)
assert(todo.getLastEntry().text == "Test Ünicode hàndling in an entry can't believe I didn't test this at first 🤣")
