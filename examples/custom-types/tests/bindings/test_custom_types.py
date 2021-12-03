from custom_types import *

# Get the custom types and check their data
val = get_custom_types_demo(None)
assert val.url == 'http://example.com/'
assert val.handle == 123

# Change some data and ensure that the round-trip works
val.url = 'http://new.example.com/'
val.handle = 456
assert val == get_custom_types_demo(val)
