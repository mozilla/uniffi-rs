/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.uniffi_fixture_rename.*;

// Test renamed types - should be accessible by their custom names
val record = RenamedRecord(renamedField = 42)
assert(record.renamedField == 42)

val enum1 = RenamedEnum.RenamedVariant
val enum2 = RenamedEnum.Record(record)

// Test renamed function
val result = renamedFunction(record)
assert(result is RenamedEnum.Record)
assert((result as RenamedEnum.Record).v1.renamedField == 42)

// Test renamed object with renamed constructor and method
val obj = RenamedObject.renamedConstructor(123)
assert(obj is RenamedObject)
val methodResult = obj.renamedMethod()
assert(methodResult == 123)

// Test trait method renaming (trait itself keeps original name)
val traitImpl = createTraitImpl(5)
assert(traitImpl.renamedTraitMethod(10) == 50) // 10 * 5 = 50

//
// Test TOML-based renaming with dot notation
//
val ktRecord = KtRecord(kotlinItem = 123)  // Field renamed: item -> kotlinItem
assert(ktRecord.kotlinItem == 123)

// Function binding_function -> ktFunction, arg record -> kotlinRecord
val ktResult = ktFunction(kotlinRecord = ktRecord)
assert(ktResult is KtEnum.KotlinRecord)

val withFields = KtEnumWithFields.KotlinVariantA(kotlinInt = 1U)

// throwing renamed exception.
try {
    ktFunction(null)
    assert(false) // Should have thrown
} catch (e: KtException.KotlinSimple) {
}

val ktObj = KtObject(100)
ktObj.kotlinMethod(kotlinArg = 10)  // method -> kotlinMethod, arg -> kotlinArg

// Test enum renaming with dot notation for variants
val ktEnum1 = KtEnum.KotlinVariantA  // Variant renamed: VariantA -> KotlinVariantA
val ktEnum2 = KtEnum.KotlinRecord(ktRecord)  // Variant renamed: Record -> KotlinRecord
assert(ktEnum1 is KtEnum.KotlinVariantA)
assert(ktEnum2 is KtEnum.KotlinRecord)

// Test error renaming - verify the error class exists with renamed variants
// Note: In Kotlin, errors become exception classes with "Exception" suffix
// So BindingError -> KtError becomes KtException with KotlinSimple variant

// Test callback interface (trait) renaming.
val ktTraitImpl = createBindingTraitImpl(3)
assert(ktTraitImpl.kotlinTraitMethod(4) == 12)
