import uniffi.traits.*

for (button in getButtons()) {
    val name = button.name()
    // Check that the name is one of the expected values
    assert(name in listOf("go", "stop"))
    // Check that we can round-trip the button through Rust
    assert(press(button).name() == name)
}

// Test a button implemented in Kotlin
class KtButton : Button {
    override fun name() = "KtButton"
}

assert(press(KtButton()).name() == "KtButton")
