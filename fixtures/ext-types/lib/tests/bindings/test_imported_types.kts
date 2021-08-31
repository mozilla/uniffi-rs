import uniffi.uniffi_one.*
import uniffi.imported_types_lib.UniffiOneType
import uniffi.imported_types_lib.*

assert(getUniffiOneType(UniffiOneType("test")).sval == "test - test")

val ct = getCombinedType(null)
assert(ct.uot.sval == "hello")
assert(ct.guid == Guid("a-guid"))
assert(ct.json == JsonObject("{\"hello\":\"there\"}"))

val ct2 = getCombinedType(ct)
assert(ct == ct2)

