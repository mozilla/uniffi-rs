/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import imported_types_lib
import Foundation

// First step: implement a trait from an external crate in Swift and pass it to a function from this
// crate.  This tests #2343 -- the codegen for this module needs to initialize the vtable from
// uniffi_one.
final class SwiftUniffiOneImpl: UniffiOneTrait {
    func hello() -> String {
        "Hello from Swift"
    }
}
assert(invokeUniffiOneTrait(t: SwiftUniffiOneImpl()) == "Hello from Swift")

let ct = getCombinedType(value: nil)
assert(ct.uot.sval == "hello")
assert(ct.guid ==  "a-guid")
assert(ct.url ==  URL(string: "http://example.com/"))

let ct2 = getCombinedType(value: ct)
assert(ct == ct2)

let t = getTraitImpl()
assert(t.hello() == "sub-lib trait impl says hello")
let sub = SubLibType(maybeEnum: nil, maybeTrait: t, maybeInterface: nil)
assert(getSubType(existing: sub).maybeTrait != nil)

let ob = ObjectsType(maybeTrait: t, maybeInterface: nil, sub: sub)
assert(getObjectsType(value: nil).maybeInterface == nil)
assert(getObjectsType(value: ob).maybeTrait != nil)
assert(getUniffiOneTrait(t: nil) == nil)

let url = URL(string: "http://example.com/")!;
assert(getUrl(url: url) ==  url)
assert(getMaybeUrl(url: url)! == url)
assert(getMaybeUrl(url: nil) == nil)
assert(getUrls(urls: [url]) == [url])
assert(getMaybeUrls(urls: [url, nil]) == [url, nil])

assert(getGuid(value: "guid") ==  "guid")
assert(getOuid(ouid: "ouid") ==  "ouid")
assert(getImportedOuid(ouid: "ouid") ==  "ouid")
assert(getNestedOuid(nouid: "ouid") ==  "ouid")
assert(getImportedNestedGuid(guid: nil) == "nested")
assert(getNestedExternalOuid(ouid: nil) == "nested-external-ouid")
assert(getImportedHandleU8(h: nil) == 3)

assert(getUniffiOneType(t: UniffiOneType(sval: "hello")).sval == "hello")
assert(getMaybeUniffiOneType(t: UniffiOneType(sval: "hello"))!.sval == "hello")
assert(getMaybeUniffiOneType(t: nil) == nil)
assert(getUniffiOneTypes(ts: [UniffiOneType(sval: "hello")]) == [UniffiOneType(sval: "hello")])
assert(getMaybeUniffiOneTypes(ts: [UniffiOneType(sval: "hello"), nil]) == [UniffiOneType(sval: "hello"), nil])
assert(getMyProcMacroType(t: UniffiOneProcMacroType(sval: "proc-macros")).sval == "proc-macros")

assert(getUniffiOneEnum(e: UniffiOneEnum.one) == UniffiOneEnum.one)
assert(getMaybeUniffiOneEnum(e: UniffiOneEnum.one)! == UniffiOneEnum.one)
assert(getMaybeUniffiOneEnum(e: nil) == nil)
assert(getUniffiOneEnums(es: [UniffiOneEnum.one]) == [UniffiOneEnum.one])
assert(getMaybeUniffiOneEnums(es: [UniffiOneEnum.one, nil]) == [UniffiOneEnum.one, nil])

do {
    try throwUniffiOneError()
    fatalError("Should have thrown")
} catch let e as UniffiOneError {
    if case let .Oops(reason) = e {
        assert(reason == "oh no")
    } else {
        fatalError("wrong error variant: \(e)")
    }
}

var counter = DispatchGroup()
counter.enter()
Task {
    do {
        try await throwUniffiOneErrorAsync()
        fatalError("Should have thrown")
    } catch let e as UniffiOneError {
        if case let .Oops(reason) = e {
            assert(reason == "oh no - async")
        } else {
            fatalError("wrong error variant: \(e)")
        }
    }
    counter.leave()
}
counter.wait()

do {
    try throwUniffiOneErrorInterface()
    fatalError("Should have thrown")
} catch let e as UniffiOneErrorInterface {
    assert(e.message() == "interface oops")
}

assert(ct.ecd.sval == "ecd")
assert(getExternalCrateInterface(val: "foo").value() == "foo")
