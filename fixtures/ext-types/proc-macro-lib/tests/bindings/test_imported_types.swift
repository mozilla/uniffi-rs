/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import imported_types_lib
import Foundation

let ct = getCombinedType(value: nil)
assert(ct.uot.sval == "hello")
assert(ct.guid ==  "a-guid")
assert(ct.url ==  URL(string: "http://example.com/"))

let ct2 = getCombinedType(value: ct)
assert(ct == ct2)

assert(getObjectsType(value: nil).maybeInterface == nil)
assert(getObjectsType(value: nil).maybeTrait == nil)
assert(getUniffiOneTrait(t: nil) == nil)

let url = URL(string: "http://example.com/")!;
assert(getUrl(url: url) ==  url)
assert(getMaybeUrl(url: url)! == url)
assert(getMaybeUrl(url: nil) == nil)
assert(getUrls(urls: [url]) == [url])
assert(getMaybeUrls(urls: [url, nil]) == [url, nil])

assert(getUniffiOneType(t: UniffiOneType(sval: "hello")).sval == "hello")
assert(getMaybeUniffiOneType(t: UniffiOneType(sval: "hello"))!.sval == "hello")
assert(getMaybeUniffiOneType(t: nil) == nil)
assert(getUniffiOneTypes(ts: [UniffiOneType(sval: "hello")]) == [UniffiOneType(sval: "hello")])
assert(getMaybeUniffiOneTypes(ts: [UniffiOneType(sval: "hello"), nil]) == [UniffiOneType(sval: "hello"), nil])

var counter = DispatchGroup()
counter.enter()
Task {
    // This async function comes from the `uniffi-one` crate
    let uniffiOneEnum = await getUniffiOneAsync()
    assert(uniffiOneEnum == UniffiOneEnum.one)

    // This async function comes from the `proc-macro-lib` crate
    let uniffiOneType = await getUniffiOneTypeAsync(t: UniffiOneType(sval: "hello"))
    assert(uniffiOneType.sval == "hello")

    counter.leave()
}
counter.wait()


assert(getUniffiOneProcMacroType(t: UniffiOneProcMacroType(sval: "hello from proc-macro world")).sval == "hello from proc-macro world")
assert(getMyProcMacroType(t: UniffiOneProcMacroType(sval: "proc-macros all the way down")).sval == "proc-macros all the way down")

assert(getUniffiOneEnum(e: UniffiOneEnum.one) == UniffiOneEnum.one)
assert(getMaybeUniffiOneEnum(e: UniffiOneEnum.one)! == UniffiOneEnum.one)
assert(getMaybeUniffiOneEnum(e: nil) == nil)
assert(getUniffiOneEnums(es: [UniffiOneEnum.one]) == [UniffiOneEnum.one])
assert(getMaybeUniffiOneEnums(es: [UniffiOneEnum.one, nil]) == [UniffiOneEnum.one, nil])

let g = getGuidProcmacro(g: nil)
assert(g == getGuidProcmacro(g: g))
