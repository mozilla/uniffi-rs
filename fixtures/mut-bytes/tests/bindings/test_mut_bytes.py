# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_mut_bytes import *

# Zero-copy &mut [u8] via UDL [ByMutRef]. Rust writes land in place.
buf = bytearray(4)
fill_bytes_udl(buf)
assert buf == bytearray([0, 1, 2, 3]), buf

inc = bytearray([1, 2, 3])
increment_bytes_udl(inc)
assert inc == bytearray([2, 3, 4]), inc

# Empty buffer is handled without crashing.
empty = bytearray(0)
fill_bytes_udl(empty)
assert empty == bytearray(0), empty
