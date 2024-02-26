# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_simple_fns import *
import typing

assert get_string() == "String created by Rust"
assert get_int() == 1289
assert string_identity("String created by Python") == "String created by Python"
assert byte_to_u32(255) == 255

a_set = new_set()
add_to_set(a_set, "foo")
add_to_set(a_set, "bar")
assert set_contains(a_set, "foo")
assert set_contains(a_set, "bar")
assert not set_contains(a_set, "baz")

assert hash_map_identity({"a": "b"}) == {"a": "b"}
assert typing.get_type_hints(hash_map_identity) == {
    "h": dict[str, str],
    "return": dict[str, str],
}
