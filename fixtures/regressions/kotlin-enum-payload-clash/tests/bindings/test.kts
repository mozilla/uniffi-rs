import uniffi.regression_kotlin_enum_payload_clash.*

// Test basic enum with name clash
val value = getValue()
when (value) {
    is AmountOrMax.Amount -> check(value.amount.value == 100UL)
    is AmountOrMax.Max -> error("Expected Amount, got Max")
}

// Test complex nested types to cover fully_qualified_type_label paths
val complexValue = getComplexValue()
when (complexValue) {
    is ComplexValue.Amounts -> {
        check(complexValue.amounts.size == 2)
        check(complexValue.amounts[0].value == 1UL)
        check(complexValue.amounts[1].value == 2UL)
    }
    else -> error("Expected Amounts variant")
}

// Test Optional type path
val optionalAmount = ComplexValue.OptionalAmount(Amount(42UL))
when (optionalAmount) {
    is ComplexValue.OptionalAmount -> check(optionalAmount.maybeAmount?.value == 42UL)
    else -> error("Expected OptionalAmount")
}

// Test Map type path
val amountMap = ComplexValue.AmountMap(mapOf("key1" to Amount(10UL), "key2" to Amount(20UL)))
when (amountMap) {
    is ComplexValue.AmountMap -> {
        check(amountMap.amountMap["key1"]?.value == 10UL)
        check(amountMap.amountMap["key2"]?.value == 20UL)
    }
    else -> error("Expected AmountMap")
}

// Test deeply nested Optional<Sequence>
val nestedOptional = ComplexValue.NestedOptional(listOf(Amount(1UL), Amount(2UL)))
when (nestedOptional) {
    is ComplexValue.NestedOptional -> {
        check(nestedOptional.maybeAmounts?.size == 2)
    }
    else -> error("Expected NestedOptional")
}

// Test deeply nested Map<String, Sequence>
val nestedMap = ComplexValue.NestedMap(
    mapOf(
        "list1" to listOf(Amount(1UL), Amount(2UL)),
        "list2" to listOf(Amount(3UL))
    )
)
when (nestedMap) {
    is ComplexValue.NestedMap -> {
        check(nestedMap.nestedMap["list1"]?.size == 2)
        check(nestedMap.nestedMap["list2"]?.size == 1)
    }
    else -> error("Expected NestedMap")
}
