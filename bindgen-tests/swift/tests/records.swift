/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

assert(roundtripSimpleRec(rec: SimpleRec(a: 42)) == SimpleRec(a: 42))
assert(RecWithDefault().a == 42)
assert(RecWithDefault(a: 10).a == 10)
assert(
  roundtripComplexRec(
    rec: ComplexRec(
      fieldU8: 0,
      fieldI8: -1,
      fieldU16: 2,
      fieldI16: -3,
      fieldU32: 4,
      fieldI32: -5,
      fieldU64: 6,
      fieldI64: -7,
      fieldF32: 8.5,
      fieldF64: 9.5,
      fieldRec: SimpleRec(a: 42)
    )
  ) == ComplexRec(
    fieldU8: 0,
    fieldI8: -1,
    fieldU16: 2,
    fieldI16: -3,
    fieldU32: 4,
    fieldI32: -5,
    fieldU64: 6,
    fieldI64: -7,
    fieldF32: 8.5,
    fieldF64: 9.5,
    fieldRec: SimpleRec(a: 42)
  )
)
