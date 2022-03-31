from custom_types import *

# Get the custom types and check their data
val = get_custom_types_demo(None)
assert val.url.scheme == 'http'
assert val.url.netloc == 'example.com'
assert val.url.path == '/'
assert val.handle == 123

# Change some data and ensure that the round-trip works
val.url = val.url._replace(netloc='new.example.com')
val.handle = 456
assert val == get_custom_types_demo(val)
