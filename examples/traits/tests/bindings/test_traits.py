from traits import *

for button in get_buttons():
    if button.name() in ["go", "stop"]:
        press(button)
    else:
        print("unknown button", button)
