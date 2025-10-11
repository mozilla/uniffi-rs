import uniffi.regression_kotlin_enum_payload_clash.*

val value = getValue()
when (value) {
    is AmountOrMax.Amount -> check(value.amount.value == 100UL)
    is AmountOrMax.Max -> error("Expected Amount, got Max")
}
