import traits

for button in getButtons() {
    let name = button.name()
    // Check that the name is one of the expected values
    assert(["go", "stop"].contains(name))
    // Check that we can round-trip the button through Rust
    assert(press(button: button).name() == name)
}

// Test a Button implemented in Swift
final class SwiftButton: Button {
    func name() -> String {
        return "SwiftButton"
    }
}

assert(press(button: SwiftButton()).name() == "SwiftButton")
