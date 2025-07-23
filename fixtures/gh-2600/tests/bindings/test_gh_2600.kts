/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.gh_2600.*;

var obj = MyStruct256()
assert(dropCount() == 0.toUInt())
obj.method()
assert(dropCount() == 0.toUInt())
