# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_simple_fns import *

assert get_string() == "String created by Rust"
assert get_int() == 1289
assert string_identity("String created by Python") == "String created by Python"
assert byte_to_u32(255) == 255
