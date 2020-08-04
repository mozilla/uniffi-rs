import uniffi.todolist.*

val todo = TodoList()

// This throws an exception:
try {
    todo.getLast()
    throw RuntimeException("Should have thrown a TodoError!")
} catch (e: TodoErrorException) {
    // It's okay, we don't have any items yet!
}

try {
    createEntryWith("")
    throw RuntimeException("Should have thrown a TodoError!")
} catch (e: TodoErrorException) {
    // It's okay, the string was empty!
}

todo.addItem("Write strings support")

assert(todo.getLast() == "Write strings support")

todo.addItem("Write tests for strings support")

assert(todo.getLast() == "Write tests for strings support")

val entry = createEntryWith("Write bindings for strings as record members")

todo.addEntry(entry)
assert(todo.getLast() == "Write bindings for strings as record members")
assert(todo.getLastEntry().text == "Write bindings for strings as record members")

todo.addItem("Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")
assert(todo.getLast() == "Test Ünicode hàndling without an entry can't believe I didn't test this at first 🤣")

val entry2 = TodoEntry("Test Ünicode hàndling in an entry can't believe I didn't test this at first 🤣")
todo.addEntry(entry2)
assert(todo.getLastEntry().text == "Test Ünicode hàndling in an entry can't believe I didn't test this at first 🤣")

assert(todo.getEntries().size == 5)

todo.addEntries(listOf(TodoEntry("foo"), TodoEntry("bar")))
assert(todo.getEntries().size == 7)
assert(todo.getLastEntry().text == "bar")

todo.addItems(listOf("bobo", "fofo"))
assert(todo.getItems().size == 9)
assert(todo.getItems()[7] == "bobo")
