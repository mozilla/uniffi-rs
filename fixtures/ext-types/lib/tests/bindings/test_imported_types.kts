/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.imported_types_lib.*
import uniffi.imported_types_sublib.*
import uniffi.uniffi_one_ns.*
import uniffi.ext_types_custom.*

val ct = getCombinedType(null)
assert(ct.uot.sval == "hello")
assert(ct.guid ==  "a-guid")
assert(ct.url ==  java.net.URL("http://example.com/"))

val ct2 = getCombinedType(ct)
assert(ct == ct2)

assert(getObjectsType(null).maybeInterface == null)
assert(getObjectsType(null).maybeTrait == null)
assert(getUniffiOneTrait(null) == null)

assert(getSubType(null).maybeInterface == null)
assert(getTraitImpl().hello() == "sub-lib trait impl says hello")

val url = java.net.URL("http://example.com/")
assert(getUrl(url) ==  url)
assert(getMaybeUrl(url)!! ==  url)
assert(getMaybeUrl(null) ==  null)
assert(getUrls(listOf(url)) ==  listOf(url))
assert(getMaybeUrls(listOf(url, null)) == listOf(url, null))

assert(getGuid("guid") == "guid")
assert(getOuid("ouid") == "ouid")
//assert(getImportedGuid("guid") == "guid")
assert(getImportedOuid("ouid") == "ouid")

val uot = UniffiOneType("hello")
assert(getUniffiOneType(uot) == uot)
assert(getMaybeUniffiOneType(uot)!! == uot)
assert(getMaybeUniffiOneType(null) == null)
assert(getUniffiOneTypes(listOf(uot)) == listOf(uot))
assert(getMaybeUniffiOneTypes(listOf(uot, null)) == listOf(uot, null))

val uopmt = UniffiOneProcMacroType("hello from proc-macro world")
assert(getUniffiOneProcMacroType(uopmt) == uopmt)
assert(getMyProcMacroType(uopmt) == uopmt)

val uoe = UniffiOneEnum.ONE
assert(getUniffiOneEnum(uoe) == uoe)
assert(getMaybeUniffiOneEnum(uoe)!! == uoe)
assert(getMaybeUniffiOneEnum(null) == null)
assert(getUniffiOneEnums(listOf(uoe)) == listOf(uoe))
assert(getMaybeUniffiOneEnums(listOf(uoe, null)) == listOf(uoe, null))

assert(ct.ecd.sval == "ecd")
assert(getExternalCrateInterface("foo").value() == "foo")
