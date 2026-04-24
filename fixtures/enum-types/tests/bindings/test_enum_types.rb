# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'enum_types'

class TestEnumTypes < Test::Unit::TestCase

  def test_animals
    # Plain enum: Doc=0, Cat=1 (sequential, starting at 0)
    assert_equal 0, EnumTypes::Animal::DOG
    assert_equal 1, EnumTypes::Animal::CAT

    assert_equal EnumTypes::Animal::DOG, EnumTypes.get_animal(nil)

    # In Ruby flat enums are integers; Animal::CAT == 1, so get_animal(1) succeeds.
    assert_equal EnumTypes::Animal::CAT, EnumTypes.get_animal(1)

    # AnimalNoReprInt
    assert_equal 3, EnumTypes::AnimalNoReprInt::DOG
    assert_equal 4, EnumTypes::AnimalNoReprInt::CAT

    # AnimalUInt: repr u8
    assert_equal 3, EnumTypes::AnimalUInt::DOG
    assert_equal 4, EnumTypes::AnimalUInt::CAT

    # AnimalLargeUInt: repr u64, Dog=u32::MAX+3, Cat=u32::MAX+4
    assert_equal 4294967295 + 3, EnumTypes::AnimalLargeUInt::DOG
    assert_equal 4294967295 + 4, EnumTypes::AnimalLargeUInt::CAT

    # AnimalSignedInt: repr i8, Dog=-3, Cat=-2, Koala=-1, Wallaby=0, Wombat=1
    assert_equal -3, EnumTypes::AnimalSignedInt::DOG
    assert_equal -2, EnumTypes::AnimalSignedInt::CAT
    assert_equal -1, EnumTypes::AnimalSignedInt::KOALA
    assert_equal 0, EnumTypes::AnimalSignedInt::WALLABY
    assert_equal 1, EnumTypes::AnimalSignedInt::WOMBAT
  end

  def test_containers
    dog_enum = EnumTypes.get_animal_enum(EnumTypes::Animal::DOG)
    assert dog_enum.dog?
    assert_equal 'dog', dog_enum[0].get_record.name

    cat_enum = EnumTypes.get_animal_enum(EnumTypes::Animal::CAT)
    assert cat_enum.cat?
    assert_equal 'cat', cat_enum[0].name

    # Equality: Ruby doesn't support equality for enums, so we can't test that dog_enum ==
    # EnumTypes.get_animal_enum(EnumTypes::Animal::DOG).
    #
    # assert dog_enum == EnumTypes.get_animal_enum(EnumTypes::Animal::DOG)
    # assert cat_enum == EnumTypes.get_animal_enum(EnumTypes::Animal::CAT)

    # Different variants are not equal.
    assert_not_equal dog_enum, cat_enum
  end

  def test_defaults
    e = EnumTypes::NamedEnumWithDefaults::I.new
    assert_equal 0, e.d
    assert_equal 1, e.e

    e2 = EnumTypes::NamedEnumWithDefaults::I.new(e: 2)
    assert_equal 0, e2.d
    assert_equal 2, e2.e
  end

  def test_boxed_types
    boxed_enum = EnumTypes.create_boxed_enum('hello')
    assert boxed_enum.boxed?
    assert_equal 'hello', EnumTypes.get_boxed_enum_value(boxed_enum)

    empty_enum = EnumTypes::EnumWithBoxedVariant::EMPTY.new
    assert empty_enum.empty?
    assert_equal 'empty', EnumTypes.get_boxed_enum_value(empty_enum)

    content = EnumTypes::BoxedContent.new(value: 'world')
    result = EnumTypes.roundtrip_boxed_record(content)
    assert_equal 'world', result.value
  end
end
