from wrapper_types import *

val = get_wrapped_types_demo(None)
assert(val.json == '{"demo":"string"}')
assert(val.handle == 123)

val = WrappedTypesDemo('{"foo":"bar"}', 456)
val2 = get_wrapped_types_demo(val)
assert(val == val2)

assert(get_next_handle(val.handle) == 457)
assert(objectify("outer", '{"hello":"there"}') == '{"outer":{"hello":"there"}}')
