/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import codable_test

// SimpleRecord
let simpleRecord = SimpleRecord(
    string: "Test",
    boolean: true,
    integer: 1,
    floatVar: 1.1,
    vec: [true]
)

let jsonSimpleRecord = try JSONEncoder().encode(simpleRecord)
let deserializedSimpleRecord = try JSONDecoder().decode(SimpleRecord.self, from: jsonSimpleRecord)
assert(deserializedSimpleRecord.string == "Test")
assert(deserializedSimpleRecord.boolean)
assert(deserializedSimpleRecord.integer == 1)
assert(deserializedSimpleRecord.floatVar == 1.1)
assert(deserializedSimpleRecord.vec == [true])

// MultiLayerRecord
let multilayer = MultiLayerRecord(
    simpleEnum: SimpleEnum.one,
    reprU8: ReprU8.two,
    simpleRecord: simpleRecord
)

let jsonMultiLayerRecord = try JSONEncoder().encode(multilayer)
let deserializedMultiLayerRecord = try JSONDecoder().decode(MultiLayerRecord.self, from: jsonMultiLayerRecord)
assert(deserializedMultiLayerRecord.simpleEnum == .one)
assert(deserializedMultiLayerRecord.reprU8 == .two)
assert(deserializedMultiLayerRecord.simpleRecord == simpleRecord)

// RecordWithOptionals
let optionals = RecordWithOptionals(
    string: nil,
    boolean: nil,
    integer: nil,
    floatVar: nil,
    vec: ["A", "B"]
)

let jsonRecordWithOptionals = try JSONEncoder().encode(optionals)
assert(String(data: jsonRecordWithOptionals, encoding: .utf8) == "{\"vec\":[\"A\",\"B\"]}")

// ComplexEnum
let complexEnum = ComplexEnum.string("test")
let jsonComplexEnum = try JSONEncoder().encode(complexEnum)
let deserializedComplexEnum = try JSONDecoder().decode(ComplexEnum.self, from: jsonComplexEnum)
assert(deserializedComplexEnum == .string("test"))

// SimpleError
let simpleError = SimpleError.OsError
let jsonSimpleError = try JSONEncoder().encode(simpleError)
let deserializedSimpleError = try JSONDecoder().decode(SimpleError.self, from: jsonSimpleError)
assert(deserializedSimpleError == .OsError)
