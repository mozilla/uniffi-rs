/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import kotlinx.serialization.*
import kotlinx.serialization.json.*
import uniffi.codable_test.*

val simpleRecord = SimpleRecord(
    string = "Test",
    boolean = true,
    integer = 1,
    floatVar = 1.1,
    vec = listOf(true),
)

val jsonSimpleRecord = Json.encodeToString(simpleRecord)
val deserializedSimpleRecord = Json.decodeFromString<SimpleRecord>(jsonSimpleRecord)
assert(deserializedSimpleRecord.string == "Test")
assert(deserializedSimpleRecord.boolean)
assert(deserializedSimpleRecord.integer == 1)
assert(deserializedSimpleRecord.floatVar == 1.1)
assert(deserializedSimpleRecord.vec == listOf(true))

// MultiLayerRecord
val multilayer = MultiLayerRecord(
    simpleEnum = SimpleEnum.ONE,
    reprU8 = ReprU8.TWO,
    simpleRecord = simpleRecord,
)

val jsonMultiLayerRecord = Json.encodeToString(multilayer)
val deserializedMultiLayerRecord = Json.decodeFromString<MultiLayerRecord>(jsonMultiLayerRecord)
assert(deserializedMultiLayerRecord.simpleEnum == SimpleEnum.ONE)
assert(deserializedMultiLayerRecord.reprU8 == ReprU8.TWO)
assert(deserializedMultiLayerRecord.simpleRecord == simpleRecord)

// RecordWithOptionals
val optionals = RecordWithOptionals(
    string = null,
    boolean = null,
    integer = null,
    floatVar = null,
    vec = listOf("A", "B"),
)

val implicitNulls = Json { explicitNulls = false }
val jsonRecordWithOptionals = implicitNulls.encodeToString(optionals)
assert(jsonRecordWithOptionals == """{"vec":["A","B"]}""")

// ComplexEnum
val withClassDiscriminator = Json { classDiscriminator = "#class" }
val complexEnum = ComplexEnum.String("test")
val jsonComplexEnum = withClassDiscriminator.encodeToString<ComplexEnum>(complexEnum)
val deserializedComplexEnum = withClassDiscriminator.decodeFromString<ComplexEnum>(jsonComplexEnum)
assert(deserializedComplexEnum == ComplexEnum.String("test"))

// SimpleError: Exceptions are not serializable in Kotlin
// val simpleException = SimpleException.OsException()
// val jsonSimpleException = Json.encodeToString(simpleException)
// val deserializedSimpleException = Json.decodeFromString<SimpleException>(jsonSimpleException)
// assert(deserializedSimpleException == SimpleException.OsException())
