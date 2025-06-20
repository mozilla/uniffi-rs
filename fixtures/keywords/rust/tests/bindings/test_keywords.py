# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from datetime import datetime, timezone

# A successful import all this test really needs...
import keywords_rust
# but might as well call something.
keywords_rust._if(0)

# python/mypy tests - not keywords, but type names in strange places.
tn = keywords_rust.get_type_names()
assert(tn.bool == False)
assert(tn.str == "")
# etc - no need to check all the values, we are just checking mypy is happy.
