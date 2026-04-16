import mutable_records

// MutableRecord should have var properties (can reassign)
var mutable = MutableRecord(value: "initial")
mutable.value = "modified"
assert(mutable.value == "modified")

// ImmutableRecord should have let properties (immutable by default)
// Just verify we can create it — attempting to assign would be a compile error
let immutable = ImmutableRecord(value: "fixed")
assert(immutable.value == "fixed")
