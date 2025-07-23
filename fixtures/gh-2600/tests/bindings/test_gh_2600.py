# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from gh_2600 import *

obj = MyStruct256()
assert(drop_count() == 0)
obj.method()
assert(drop_count() == 0)
