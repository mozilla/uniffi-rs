/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_proc_macro

let one = makeOne(inner: 123)
assert(one.inner == 123)

let two = Two(a: "a", b: nil)
assert(takeTwo(two: two) == "a")

let obj = makeObject()
assert(obj.isHeavy() == .uncertain)

assert(enumIdentity(value: .true) == .true)

// just make sure this works / doesn't crash
let three = Three(obj: obj)

do {
    try alwaysFails()
    fatalError("alwaysFails should have thrown")
} catch BasicError.OsError {
}

try! obj.doStuff(times: 5)

do {
    try obj.doStuff(times: 0)
    fatalError("doStuff should throw if its argument is 0")
} catch FlatError.InvalidInput {
}
