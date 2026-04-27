# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'trait_methods'

# Keep module-qualified names to avoid ambiguity with the TraitMethods class which has the same
# name as the module.
Tm = TraitMethods

class TestTraitMethodsObject < Test::Unit::TestCase
  def test_to_s
    m = Tm::TraitMethods.new 'yo'

    assert_equal 'TraitMethods(yo)', m.to_s
  end

  def test_inspect
    m = Tm::TraitMethods.new 'yo'

    assert_equal 'TraitMethods { val: "yo" }', m.inspect
  end

  def test_eq
    m = Tm::TraitMethods.new 'yo'

    assert_equal m, Tm::TraitMethods.new('yo')
    assert_not_equal m, Tm::TraitMethods.new('yoyo')
  end

  def test_eq_wrong_type
    m = Tm::TraitMethods.new 'yo'

    assert_not_equal m, 17
  end

  def test_hash
    m = Tm::TraitMethods.new 'm'
    h = {}
    h[m] = 'm'

    assert h.key?(m)
    assert_equal 'm', h[Tm::TraitMethods.new('m')]
  end

  def test_ord
    a = Tm::TraitMethods.new 'a'
    b = Tm::TraitMethods.new 'b'

    assert a < b
    assert a <= b
    assert a <= Tm::TraitMethods.new('a')
    assert b > a
    assert b >= a
    assert b >= Tm::TraitMethods.new('b')
  end
end

class TestProcTraitMethodsObject < Test::Unit::TestCase
  def test_to_s
    m = Tm::ProcTraitMethods.new 'yo'

    assert_equal 'ProcTraitMethods(yo)', m.to_s
  end

  def test_inspect
    m = Tm::ProcTraitMethods.new 'yo'

    assert_equal 'ProcTraitMethods { val: "yo" }', m.inspect
  end

  def test_eq
    m = Tm::ProcTraitMethods.new 'yo'

    assert_equal m, Tm::ProcTraitMethods.new('yo')
    assert_not_equal m, Tm::ProcTraitMethods.new('yoyo')
  end

  def test_eq_wrong_type
    m = Tm::ProcTraitMethods.new 'yo'

    assert_not_equal m, 17
  end

  def test_hash
    m = Tm::ProcTraitMethods.new 'm'
    h = {}
    h[m] = 'm'

    assert h.key?(m)
  end

  def test_ord
    a = Tm::ProcTraitMethods.new 'a'
    b = Tm::ProcTraitMethods.new 'b'

    assert a < b
    assert a <= b
    assert b > a
    assert b >= a
  end
end

class TestTraitRecord < Test::Unit::TestCase
  def test_inspect
    r = Tm::TraitRecord.new s: 'yo', i: 2

    assert_equal 'TraitRecord { s: "yo", i: 2 }', r.inspect
  end

  def test_eq
    # eq compares only `s`, not `i`
    assert_equal Tm::TraitRecord.new(s: 'yo', i: 2), Tm::TraitRecord.new(s: 'yo', i: 3)
    assert_not_equal Tm::TraitRecord.new(s: 'yo', i: 2), Tm::TraitRecord.new(s: 'hi', i: 2)
  end

  def test_hash
    r1 = Tm::TraitRecord.new(s: 'yo', i: 2)
    r2 = Tm::TraitRecord.new(s: 'yo', i: 3)
    h = {}
    h[r1] = 'r1'

    # same hash since hash is based on `s` only
    assert h.key?(r2)
  end

  def test_ord
    r1 = Tm::TraitRecord.new(s: 'a', i: 0)
    r2 = Tm::TraitRecord.new(s: 'b', i: 0)

    assert r1 < r2
  end
end

class TestUdlRecord < Test::Unit::TestCase
  def test_inspect
    r = Tm::UdlRecord.new s: 'yo', i: 2

    assert_equal 'UdlRecord { s: "yo", i: 2 }', r.inspect
  end

  def test_eq
    # eq compares only `s`, not `i`
    assert_equal Tm::UdlRecord.new(s: 'yo', i: 2), Tm::UdlRecord.new(s: 'yo', i: 2)
    assert_not_equal Tm::UdlRecord.new(s: 'yo', i: 2), Tm::UdlRecord.new(s: 'hi', i: 3)
  end

  def test_hash
    r1 = Tm::UdlRecord.new(s: 'yo', i: 2)
    r2 = Tm::UdlRecord.new(s: 'yo', i: 3)
    h = {}
    h[r1] = 'r1'

    # same hash since hash is based on `s` only
    assert h.key?(r2)
  end

  def test_ord
    assert Tm::UdlRecord.new(s: 'a', i: 0) < Tm::UdlRecord.new(s: 'b', i: 0)
  end
end

class TestTraitEnum < Test::Unit::TestCase
  def test_to_s
    m = Tm::TraitEnum::S.new 'yo'

    assert_equal 'TraitEnum::S("yo")', m.to_s
  end

  def test_inspect
    m = Tm::TraitEnum::S.new 'yo'

    assert_equal 'S("yo")', m.inspect
  end

  def test_eq
    assert_equal Tm::TraitEnum::S.new('1'), Tm::TraitEnum::S.new('1')

    # eq is descriminant-only: S("1") == S("2")
    assert_equal Tm::TraitEnum::S.new('1'), Tm::TraitEnum::S.new('2')
    assert_equal Tm::TraitEnum::I.new(1), Tm::TraitEnum::I.new(1)

    # different variants are never equal
    assert_not_equal Tm::TraitEnum::S.new('1'), Tm::TraitEnum::I.new(1)
  end

  def test_eq_wrong_type
    assert_not_equal Tm::TraitEnum::S.new('1'), 17
  end

  def test_hash
    m = Tm::TraitEnum::S.new('m')
    h = {}
    h[m] = 'm'

    assert h.key?(m)
  end

  def test_ord
    s = Tm::TraitEnum::S.new('1')
    i = Tm::TraitEnum::I.new(1)

    assert s < i
    assert s <= i
  end
end

class TestUdlEnum < Test::Unit::TestCase
  def test_inspect
    m = Tm::UdlEnum::S.new s: 'yo'

    assert_equal 'S { s: "yo" }', m.inspect
  end

  def test_eq
    assert_equal Tm::UdlEnum::S.new(s: 'yo'), Tm::UdlEnum::S.new(s: 'yo')

    # eq is descriminant-only
    assert_equal Tm::UdlEnum::S.new(s: '1'), Tm::UdlEnum::S.new(s: '2')
    assert_not_equal Tm::UdlEnum::S.new(s: '1'), Tm::UdlEnum::I.new(i: 1)
  end

  def test_ord
    assert Tm::UdlEnum::S.new(s: '1') < Tm::UdlEnum::I.new(i: 1)
  end
end
