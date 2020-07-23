import todolist


let todo = TodoList()
todo.add_item(todo: "Write swift bindings")
assert( todo.get_last() == "Write swift bindings")

todo.add_item(todo: "Write tests for bindings")
assert(todo.get_last() == "Write tests for bindings")

let entry = TodoEntry(text: "Write bindings for strings as record members")
todo.add_entry(entry: entry)
assert(todo.get_last() == "Write bindings for strings as record members")

todo.add_item(todo: "Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")
assert(todo.get_last() == "Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")

let entry2 = TodoEntry(text: "Test Ünicode hàndling in an entry can't believe I didn't test this at first 🤣")
todo.add_entry(entry: entry2)
assert(todo.get_last_entry() == entry2)
