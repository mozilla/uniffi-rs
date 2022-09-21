# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_simple_iface import *

obj = make_object(9000)
assert obj.get_inner() == 9000
obj.some_method()
