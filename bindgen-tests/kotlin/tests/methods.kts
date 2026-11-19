/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.uniffi_bindgen_tests.*

// Call a `describeCall` method and check the description of the receiver and each argument
fun checkDescribeCall(
    receiver: String,
    describeCall: (UByte, UByte, String, String, SimpleRec, SimpleRec, EnumNoData, EnumWithData, EnumWithData, TestInterface, TestInterface) -> String,
) {
    val description = describeCall(
        2.toUByte(),
        3.toUByte(),
        "four",
        "five",
        SimpleRec(a=6.toUByte()),
        SimpleRec(a=7.toUByte()),
        EnumNoData.B,
        EnumWithData.A(8.toUByte(), 9.toUShort()),
        EnumWithData.B("ten", 11u),
        TestInterface(12u),
        TestInterface(13u),
    )
    assert(description == "$receiver 2 3 four five rec(6) rec(7) B A(8,9) B(ten,11) interface(12) interface(13)")
}

checkDescribeCall("rec(1)", MethodTestRec(value=1.toUByte())::describeCall)
checkDescribeCall("enum_no_data(B)", MethodTestEnumNoData.B::describeCall)
checkDescribeCall("enum_with_data(A(1))", MethodTestEnumWithData.A(1.toUByte())::describeCall)
checkDescribeCall("enum_with_data(B(one))", MethodTestEnumWithData.B("one")::describeCall)
checkDescribeCall("interface(1)", MethodTestInterface(1.toUByte())::describeCall)
