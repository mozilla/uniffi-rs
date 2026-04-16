import uniffi.uniffi_bindgen_tests.*
import uniffi.uniffi_bindgen_tests_external_types_source.*

assert(roundtripExtRecord(ExternalRec(a=42.toUByte())) == ExternalRec(a=42.toUByte()))
assert(roundtripExtEnum(ExternalEnum.TWO) == ExternalEnum.TWO)

val extInterface = ExternalInterface(20u)
assert(roundtripExtInterface(extInterface).getValue() == 20u)

assert(roundtripExtCustomType(100uL) == 100uL)
