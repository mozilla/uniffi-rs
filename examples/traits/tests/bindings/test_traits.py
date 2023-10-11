from traits import *

for button in get_buttons():
    name = button.name()
    # Check that the name is one of the expected values
    assert(name in ["go", "stop"])
    # Check that we can round-trip the button through Rust
    assert(press(button).name() == name)

# Test a button implemented in Python
class PyButton(Button):
    def name(self):
        return "PyButton"

assert(press(PyButton()).name() == "PyButton")
