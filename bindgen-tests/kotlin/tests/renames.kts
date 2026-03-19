import uniffi.uniffi_bindgen_tests.*


val rec = RenamedRecord(item=42)
assert(rec.item == 42)

val enum1 = RenamedEnum.RenamedVariant
val enum2 = RenamedEnum.Record(rec)

val returnedRec = renamedFunction(rec)
when(returnedRec) {
    is RenamedEnum.Record -> assert(returnedRec.v1 == rec)
    else -> throw RuntimeException("invalid renamedFunction return: ${returnedRec}")
}

val obj = RenamedObject.renamedConstructor(123)
assert(obj.renamedMethod() == 123)

val traitImpl = createTraitImpl(5)
assert(traitImpl.renamedTraitMethod(10) == 50)

val ktRec = KtRecord(ktItem=100)
assert(ktRec.ktItem == 100)

// Test constructing BindingEnumToRename variants
KtEnum.KtVariantA
KtEnum.KtRecord(ktRec)
KtEnumWithFields.KtVariantA(1u)

val returnedKtRec = ktFunction(ktRec);
when (returnedKtRec) {
    is KtEnum.KtRecord -> assert(returnedKtRec.v1 == ktRec)
    else -> throw RuntimeException("invalid ktFunction return: ${returnedKtRec}")
}

try {
   ktFunction(null)
   throw RuntimeException("expected KtException.KtSimple")
} catch (e: KtException.KtSimple) {
    // Expected
}

val ktObj = KtObject(200)
assert(ktObj.ktMethod(50) == 250)

val ktTraitImpl = createBindingTraitToRenameImpl(3)
assert(ktTraitImpl.ktTraitMethod(7) == 21)
