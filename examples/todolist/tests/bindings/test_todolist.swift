import todolist


let todo = TodoList()
todo.addItem(todo: "Write swift bindings")
assert( todo.getLast() == "Write swift bindings")

todo.addItem(todo: "Write tests for bindings")
assert(todo.getLast() == "Write tests for bindings")

let entry = TodoEntry(text: "Write bindings for strings as record members")
todo.addEntry(entry: entry)
assert(todo.getLast() == "Write bindings for strings as record members")

todo.addItem(todo: "Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")
assert(todo.getLast() == "Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")

let entry2 = TodoEntry(text: "Test Ünicode hàndling in an entry can't believe I didn't test this at first 🤣")
todo.addEntry(entry: entry2)
assert(todo.getLastEntry() == entry2)

assert(todo.getEntries().count == 5)

todo.addEntries(entries: [TodoEntry(text: "foo"), TodoEntry(text: "bar")])
assert(todo.getEntries().count == 7)
assert(todo.getItems().count == 7)
assert(todo.getLast() == "bar")

todo.addItems(items: ["bobo", "fofo"])
assert(todo.getItems().count == 9)
assert(todo.getItems()[7] == "bobo")
