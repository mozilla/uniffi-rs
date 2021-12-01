/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import omit_argument_labels

assert(31 == noArgs())
assert(3 == oneArg(3))
assert(7 == multipleArgs(7, "look, I can omit the labels"))
