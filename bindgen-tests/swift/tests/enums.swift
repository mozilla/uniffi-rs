/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

assert(
    roundtripEnumWithData(en: EnumWithData.a(value: 10)) ==
    EnumWithData.a(value: 10));
assert(
    roundtripEnumWithData(en: EnumWithData.b(value: "Ten")) ==
    EnumWithData.b(value: "Ten"));
assert(
    roundtripEnumWithData(en: EnumWithData.c) == 
    EnumWithData.c);

assert(
    roundtripComplexEnum(en: ComplexEnum.a(value: EnumNoData.c)) ==
    ComplexEnum.a(value: EnumNoData.c))
assert(
    roundtripComplexEnum(en: ComplexEnum.b(value: EnumWithData.a(value: 20))) ==
    ComplexEnum.b(value: EnumWithData.a(value: 20)))
assert(
    roundtripComplexEnum(en: ComplexEnum.c(value: SimpleRec(a: 30))) ==
    ComplexEnum.c(value: SimpleRec(a: 30)))

// Test that the enum discriminant values

// All discriminants specified, use the specified values
assert(ExplicitValuedEnum.first.rawValue == 1);
assert(ExplicitValuedEnum.second.rawValue == 2);
assert(ExplicitValuedEnum.fourth.rawValue == 4);
assert(ExplicitValuedEnum.tenth.rawValue == 10);
assert(ExplicitValuedEnum.eleventh.rawValue == 11);
assert(ExplicitValuedEnum.thirteenth.rawValue == 13);

// Some discriminants specified, increment by one for any unspecified variants
assert(GappedEnum.one.rawValue == 10);
assert(GappedEnum.two.rawValue == 11); // Sequential value after ONE (10+1)
assert(GappedEnum.three.rawValue == 14); // Explicit value again
