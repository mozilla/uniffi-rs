/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// Call a `describeCall` method and check the description of the receiver and each argument
func checkDescribeCall(
  receiver: String,
  describeCall: (
    UInt8, UInt8, String, String, SimpleRec, SimpleRec, EnumNoData, EnumWithData, EnumWithData,
    TestInterface, TestInterface
  ) -> String
) {
  let description = describeCall(
    2,
    3,
    "four",
    "five",
    SimpleRec(a: 6),
    SimpleRec(a: 7),
    EnumNoData.b,
    EnumWithData.a(value: 8, value2: 9),
    EnumWithData.b("ten", 11),
    TestInterface(value: 12),
    TestInterface(value: 13)
  )
  assert(
    description
      == "\(receiver) 2 3 four five rec(6) rec(7) B A(8,9) B(ten,11) interface(12) interface(13)")
}

checkDescribeCall(receiver: "rec(1)", describeCall: MethodTestRec(value: 1).describeCall)
checkDescribeCall(receiver: "enum_no_data(B)", describeCall: MethodTestEnumNoData.b.describeCall)
checkDescribeCall(
  receiver: "enum_with_data(A(1))", describeCall: MethodTestEnumWithData.a(value: 1).describeCall)
checkDescribeCall(
  receiver: "enum_with_data(B(one))", describeCall: MethodTestEnumWithData.b("one").describeCall)
checkDescribeCall(receiver: "interface(1)", describeCall: MethodTestInterface(value: 1).describeCall)
