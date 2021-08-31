import uniffi.ext_types_guid.*

assert(getGuid(null) == Guid("NewGuid"))
assert(getGuid(Guid("SomeGuid")) == Guid("SomeGuid"))

val helper = getGuidHelper(null)
assert(helper.guid == Guid("first-guid"))
assert(helper.guids == listOf(Guid("second-guid"), Guid("third-guid")))
assert(helper.maybeGuid == null)
