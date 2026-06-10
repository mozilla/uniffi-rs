# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TraitImpl
  attr_accessor :value

  @@ref_count = 0

  def self.ref_count
    @@ref_count
  end

  def self.finalizer
    proc { @@ref_count -= 1 }
  end

  def initialize(value)
    @value = value
    @@ref_count += 1
    ObjectSpace.define_finalizer(self, self.class.finalizer)
  end

  def noop; end

  def get_value
    value
  end

  def set_value(value)
    self.value = value
  end

  def roundtrip_record(value)
    value
  end

  def roundtrip_enum(value)
    value
  end

  def roundtrip_interface(value)
    value
  end

  def throw_if_equal(numbers)
    raise UniffiBindgenTests::TestError::Failure1 if numbers.a == numbers.b

    numbers
  end
end

class TestTraitInterfaces < Test::Unit::TestCase
  include UniffiBindgenTests

  def check_rust_impl(trait_impl)
    trait_impl.noop
    assert_equal 42, trait_impl.get_value
    trait_impl.set_value(43)
    assert_equal 43, trait_impl.get_value
    assert_raises(TestError::Failure1) do
      trait_impl.throw_if_equal(CallbackInterfaceNumbers.new(a: 10, b: 10))
    end
    numbers = CallbackInterfaceNumbers.new(a: 10, b: 11)
    assert_equal numbers, trait_impl.throw_if_equal(numbers)

    assert_equal trait_impl.roundtrip_record(SimpleRec.new(a: 10)), SimpleRec.new(a: 10)
    assert_equal(
      trait_impl.roundtrip_enum(EnumWithData::A.new(value: 10, value2: 20)),
      EnumWithData::A.new(value: 10, value2: 20)
    )
    assert_equal trait_impl.roundtrip_interface(TestInterface.new(20)).get_value, 20
  end

  def check_rb_impl(trait_impl)
    UniffiBindgenTests.invoke_test_trait_interface_noop(trait_impl)
    assert_equal 42, UniffiBindgenTests.invoke_test_trait_interface_get_value(trait_impl)
    UniffiBindgenTests.invoke_test_trait_interface_set_value(trait_impl, 43)
    assert_equal 43, UniffiBindgenTests.invoke_test_trait_interface_get_value(trait_impl)
    assert_raises(TestError::Failure1) do
      UniffiBindgenTests.invoke_test_trait_interface_throw_if_equal(
        trait_impl,
        CallbackInterfaceNumbers.new(a: 10, b: 10)
      )
    end
    numbers = CallbackInterfaceNumbers.new(a: 10, b: 11)
    assert_equal numbers, UniffiBindgenTests.invoke_test_trait_interface_throw_if_equal(trait_impl, numbers)

    assert_equal(
      UniffiBindgenTests.invoke_test_trait_interface_roundtrip_record(trait_impl, SimpleRec.new(a: 10)),
      SimpleRec.new(a: 10)
    )
    assert_equal(
      UniffiBindgenTests.invoke_test_trait_interface_roundtrip_enum(trait_impl,
                                                                    EnumWithData::A.new(value: 10, value2: 20)),
      EnumWithData::A.new(value: 10, value2: 20)
    )

    assert_equal(
      UniffiBindgenTests.invoke_test_trait_interface_roundtrip_interface(trait_impl, TestInterface.new(20)).get_value,
      20
    )
  end

  def test_rust_impl
    check_rust_impl(UniffiBindgenTests.create_test_trait_interface(42))
  end

  def test_rust_impl_roundtripped
    check_rust_impl(
      UniffiBindgenTests.roundtrip_test_trait_interface(
        UniffiBindgenTests.create_test_trait_interface(42)
      )
    )
  end

  def test_rust_impl_roundtripped_list
    check_rust_impl(
      UniffiBindgenTests.roundtrip_test_trait_interface_list(
        [UniffiBindgenTests.create_test_trait_interface(42)]
      )[0]
    )
  end

  def test_rb_impl
    check_rb_impl(TraitImpl.new(42))
    GC.start

    assert_equal 0, TraitImpl.ref_count
  end

  def test_rb_impl_roundtripped_list
    impl = UniffiBindgenTests.roundtrip_test_trait_interface_list([TraitImpl.new(42)])[0]
    check_rb_impl(impl)
    # rubocop:disable Lint/UselessAssignment
    impl = nil
    # rubocop:enable Lint/UselessAssignment
    GC.start

    assert_equal 0, TraitImpl.ref_count
  end

  def test_rb_impl_roundtripped
    impl = UniffiBindgenTests.roundtrip_test_trait_interface(TraitImpl.new(42))
    check_rb_impl(impl)
    # rubocop:disable Lint/UselessAssignment
    impl = nil
    # rubocop:enable Lint/UselessAssignment
    GC.start

    assert_equal 0, TraitImpl.ref_count
  end
end

class TestAsyncTraitInterfaces < Test::Unit::TestCase
  include UniffiBindgenTests

  class AsyncTraitImpl < AsyncTestTraitInterface
    @@ref_count = 0

    def self.reset_ref_count
      @@ref_count = 0
    end

    def self.ref_count
      @@ref_count
    end

    def self.define_finalizer
      Proc.new { |_id| @@ref_count -= 1 }
    end

    def initialize(value)
      @value = value
      @@ref_count += 1
      ObjectSpace.define_finalizer(self, self.class.define_finalizer)
    end

    def noop
    end

    def get_value
      @value
    end

    def set_value(value)
      @value = value
    end

    def throw_if_equal(numbers)
      if numbers.a == numbers.b
        raise UniffiBindgenTests::TestError::Failure1.new
      end
      numbers
    end
  end

  def check_async_rust_impl(async_rust_trait_impl)
    async_rust_trait_impl.noop

    assert_equal 42, async_rust_trait_impl.get_value

    async_rust_trait_impl.set_value(43)
    assert_equal 43, async_rust_trait_impl.get_value

    assert_raises(TestError::Failure1) do
      async_rust_trait_impl.throw_if_equal(CallbackInterfaceNumbers.new(a: 10, b: 10))
    end

    assert_equal(
      async_rust_trait_impl.throw_if_equal(CallbackInterfaceNumbers.new(a: 10, b: 11)),
      CallbackInterfaceNumbers.new(a: 10, b: 11)
    )
  end

  def check_async_rb_impl(async_rb_trait_impl)
    UniffiBindgenTests.invoke_async_test_trait_interface_noop async_rb_trait_impl

    assert_equal 42, UniffiBindgenTests.invoke_async_test_trait_interface_get_value(async_rb_trait_impl)
    UniffiBindgenTests.invoke_async_test_trait_interface_set_value async_rb_trait_impl, 43
    assert_equal 43, UniffiBindgenTests.invoke_async_test_trait_interface_get_value(async_rb_trait_impl)

    assert_raises(TestError::Failure1) do
      async_rb_trait_impl.throw_if_equal(CallbackInterfaceNumbers.new(a: 10, b: 10))
    end

    assert_equal(
      async_rb_trait_impl.throw_if_equal(CallbackInterfaceNumbers.new(a: 10, b: 11)),
      CallbackInterfaceNumbers.new(a: 10, b: 11)
    )
  end

  def test_async_rust_impl
    check_async_rust_impl UniffiBindgenTests.create_async_test_trait_interface(42)	
  end

  def test_async_rust_impl_roundtripped
    impl = UniffiBindgenTests.roundtrip_async_test_trait_interface(
      UniffiBindgenTests.create_async_test_trait_interface(42)
    )

    check_async_rust_impl impl
  end

  def test_async_rust_impl_roundtripped_list
    impl = UniffiBindgenTests.roundtrip_async_test_trait_interface_list(
      [UniffiBindgenTests.create_async_test_trait_interface(42)]
    )[0]

    check_async_rust_impl impl
  end

  def test_async_rb_impl
    impl = AsyncTraitImpl.new(42)
    check_async_rb_impl impl
    # rubocop:disable Lint/UselessAssignment
    impl = nil
    # rubocop:enable Lint/UselessAssignment
    GC.start

    assert_equal 0, AsyncTraitImpl.ref_count
  end

  def test_async_rb_impl_roundtripped
    impl = UniffiBindgenTests.roundtrip_async_test_trait_interface(AsyncTraitImpl.new(42))
    check_async_rb_impl impl
    # rubocop:disable Lint/UselessAssignment
    impl = nil
    # rubocop:enable Lint/UselessAssignment
    GC.start

    assert_equal 0, AsyncTraitImpl.ref_count
  end

  def test_async_rb_impl_roundtripped_list
    impl = UniffiBindgenTests.roundtrip_async_test_trait_interface_list([AsyncTraitImpl.new(42)])[0]
    check_async_rb_impl impl
    # rubocop:disable Lint/UselessAssignment
    impl = nil
    # rubocop:enable Lint/UselessAssignment
    GC.start

    assert_equal 0, AsyncTraitImpl.ref_count
  end
end

class TestRustOnlyTraitInterfaces < Test::Unit::TestCase
  include UniffiBindgenTests

  def check_rust_only_impl(impl)
    assert_raises(TestError::Failure1) do
      impl.throw_if_equal(CallbackInterfaceNumbers.new(a: 10, b: 10))
    end

    assert_equal(
      impl.throw_if_equal(CallbackInterfaceNumbers.new(a: 10, b: 11)),
      CallbackInterfaceNumbers.new(a: 10, b: 11)
    )
  end

  def test_rust_only_impl
    check_rust_only_impl UniffiBindgenTests.create_rust_only_test_trait_interface
  end
end

class TestFireignOnlyTraitInterfaces < Test::Unit::TestCase
  include UniffiBindgenTests

  class ForeignOnlyImpl < TestForeignOnlyTraitInterface
    include UniffiBindgenTests

    def throw_if_equal(numbers)
      raise TestError::Failure1 if numbers.a == numbers.b

      numbers
    end
  end

  def check_foreign_only_impl(impl)
    assert_raises(TestError::Failure1) do
      UniffiBindgenTests.invoke_test_foreign_only_trait_throw_if_equal(
        impl,
        CallbackInterfaceNumbers.new(a: 10, b: 10)
      )
    end

    assert_equal(
      UniffiBindgenTests.invoke_test_foreign_only_trait_throw_if_equal(
        impl,
        CallbackInterfaceNumbers.new(a: 10, b: 11)
      ),
      CallbackInterfaceNumbers.new(a: 10, b: 11)
    )
  end

  def test_foreign_only_impl
    check_foreign_only_impl ForeignOnlyImpl.new
  end

  def test_foreign_only_impl_roundtripped
    check_foreign_only_impl UniffiBindgenTests.roundtrip_test_foreign_only_trait(ForeignOnlyImpl.new)
  end
end
