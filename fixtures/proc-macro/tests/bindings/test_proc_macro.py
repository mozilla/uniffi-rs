# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from proc_macro import *
from proc_macro import TraitWithForeignImpl

one = make_one(123)
assert one.inner == 123
assert one_inner_by_ref(one) == 123
assert one.get_inner_value() == 123

two = Two(a="a")
assert take_two(two) == "a"

rwb = RecordWithBytes(some_bytes=bytes([1,2,3]))
assert take_record_with_bytes(rwb) == bytes([1,2,3])

obj = Object()
obj = Object.named_ctor(1)
assert obj.is_heavy() == MaybeBool.UNCERTAIN
obj2 = Object()
assert obj.is_other_heavy(obj2) == MaybeBool.UNCERTAIN

robj = Renamed()
assert(robj.func())
assert(rename_test())

trait_impl = obj.get_trait(None)
assert isinstance(trait_impl, Trait)
assert trait_impl.concat_strings("foo", "bar") == "foobar"
assert obj.get_trait(trait_impl).concat_strings("foo", "bar") == "foobar"
assert concat_strings_by_ref(trait_impl, "foo", "bar") == "foobar"
assert StructWithTrait("me").concat_strings("foo", "bar") == "me: foobar"

trait_impl2 = obj.get_trait_with_foreign(None)
# This is an instance of `TraitWithForeignImpl` - `TraitWithForeign` is used primarily for subclassing.
assert isinstance(trait_impl2, TraitWithForeignImpl)
assert trait_impl2.name() == "RustTraitImpl"
assert obj.get_trait_with_foreign(trait_impl2).name() == "RustTraitImpl"

assert enum_identity(MaybeBool.TRUE) == MaybeBool.TRUE

# just make sure this works / doesn't crash
three = Three(obj=obj)

assert(make_zero().inner == "ZERO")
assert(make_record_with_bytes().some_bytes == bytes([0, 1, 2, 3, 4]))

assert(make_hashmap(1, 2) == {1: 2})
d = {1: 2}
assert(return_hashmap(d) == d)

assert(join(["a", "b", "c"], ":") == "a:b:c")

try:
    always_fails()
except BasicError.OsError:
    pass
else:
    raise Exception("always_fails should have thrown")

obj.do_stuff(5)

try:
    obj.do_stuff(0)
except FlatError.InvalidInput:
    pass
else:
    raise Exception("do_stuff should throw if its argument is 0")


# Defaults

record_with_defaults = RecordWithDefaults(no_default_string="Test")
assert(record_with_defaults.no_default_string == "Test")
assert(record_with_defaults.boolean == True)
assert(record_with_defaults.integer == 42)
assert(record_with_defaults.float_var == 4.2)
assert(record_with_defaults.vec == [])
assert(record_with_defaults.opt_vec == None)
assert(record_with_defaults.opt_integer == 42)
assert(record_with_defaults.custom_integer == 42)

assert(RecordWithDefaults(no_default_string="", vec=["oops"]) == RecordWithDefaults(no_default_string="", vec=["oops"]))

record_with_implicit_defaults = RecordWithImplicitDefaults()
assert(record_with_implicit_defaults.boolean == False)
assert(record_with_implicit_defaults.int8 == 0)
assert(record_with_implicit_defaults.uint8 == 0)
assert(record_with_implicit_defaults.int16 == 0)
assert(record_with_implicit_defaults.uint16 == 0)
assert(record_with_implicit_defaults.int32 == 0)
assert(record_with_implicit_defaults.uint32 == 0)
assert(record_with_implicit_defaults.int64 == 0)
assert(record_with_implicit_defaults.uint64 == 0)
assert(record_with_implicit_defaults.afloat == 0.0)
assert(record_with_implicit_defaults.adouble == 0.0)
assert(record_with_implicit_defaults.vec == [])
assert(record_with_implicit_defaults.map == dict())
assert(record_with_implicit_defaults.some_bytes == b'')
assert(record_with_implicit_defaults.opt_int32 is None)
assert(record_with_implicit_defaults.custom_integer == 0)

assert(double_with_default() == 42)
assert(sum_with_default(1) == 1)
assert(sum_with_default(1, 2) == 3)

obj_with_defaults = ObjectWithDefaults()
assert(obj_with_defaults.add_to_num() == 42)
assert(obj_with_defaults.add_to_implicit_num() == 30)
assert(obj_with_defaults.add_to_implicit_num(1) == 31)

# Traits
class PyTestCallbackInterface(TestCallbackInterface):
    def do_nothing(self):
        pass

    def add(self, a, b):
        return a + b

    def optional(self, a):
        if a is None:
            return 0
        return a

    def with_bytes(self, rwb):
        return rwb.some_bytes

    def try_parse_int(self, value):
        if value == "force-unexpected-error":
            # raise an error that's not expected
            raise KeyError(value)
        try:
            return int(value)
        except BaseException:
            raise BasicError.InvalidInput()

    def callback_handler(self, h):
        v = h.take_error(BasicError.InvalidInput())
        return v

    def get_other_callback_interface(self):
        return PyTestCallbackInterface2()

class PyTestCallbackInterface2(OtherCallbackInterface):
    def multiply(self, a, b):
        return a * b

call_callback_interface(PyTestCallbackInterface())

# udl exposed functions with procmacro types.
assert get_one(None).inner == 0
assert get_bool(None) == MaybeBool.UNCERTAIN
assert get_object(None).is_heavy() == MaybeBool.UNCERTAIN
assert get_trait_with_foreign(None).name() == "RustTraitImpl"
assert get_externals(None).one is None

# values for enums without an explicit value are their index.
assert(MaybeBool.TRUE.value == 0)
assert(MaybeBool.FALSE.value == 1)
assert(MaybeBool.UNCERTAIN.value == 2)
# values with an explicit value should be that value.
assert(ReprU8.ONE.value == 1)
assert(ReprU8.THREE.value == 3)
#assert(ReprU8.FIVE.value == 5)

assert(get_mixed_enum(None) == MixedEnum.INT(1))
assert(get_mixed_enum(MixedEnum.NONE()) == MixedEnum.NONE())
assert(get_mixed_enum(MixedEnum.STRING("hello")) == MixedEnum.STRING("hello"))
assert(MixedEnum.STRING("hello")[0] == "hello")
assert(str(MixedEnum.STRING("hello")) == "MixedEnum.STRING('hello',)")

assert(MixedEnum.BOTH("hello", 1)[0] == "hello")
assert(MixedEnum.BOTH("hello", 1)[1] == 1)
assert(MixedEnum.BOTH("hello", 1)[:] == ('hello', 1))
assert(MixedEnum.BOTH("hello", 1)[-1] == 1)
assert(str(MixedEnum.BOTH("hello", 2)) == "MixedEnum.BOTH('hello', 2)")
assert(MixedEnum.STRING("").is_not_none())
assert(not MixedEnum.NONE().is_not_none())

# In #2270 we realized confusion about whether we generated our
# variant checker as, eg, `is_ALL()` vs `is_all()` so decided to do both.
assert(get_mixed_enum(MixedEnum.ALL("string", 2)).is_all())
assert(get_mixed_enum(MixedEnum.ALL("string", 2)).is_ALL())

assert(get_mixed_enum(MixedEnum.VEC(["string"])).is_VEC())
