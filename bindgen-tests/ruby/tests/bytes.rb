# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestBytes < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_roundtrip_bytes
    data = 'test-data'.b

    assert_equal data, UniffiBindgenTests.roundtrip_bytes(data)
  end

  def test_roundtrip_empty_bytes
    assert_equal ''.b, UniffiBindgenTests.roundtrip_bytes(''.b)
  end

  def test_roundtrip_binary_bytes
    data = "\x00\x01\x02\xFF".b

    assert_equal data, UniffiBindgenTests.roundtrip_bytes(data)
  end

  def test_bytes_encoding
    result = UniffiBindgenTests.roundtrip_bytes('hello'.b)

    assert_equal Encoding::BINARY, result.encoding
  end

  # Zero-copy &[u8] — proc-macro path
  def zero_copy_bytes
    assert_equal 0, sum_bytes_procmacro(''.b)
    assert_equal 6, sum_bytes_procmacro("\x01\x02\x03".b)
    assert_nil first_byte_procmacro(''.b)
    assert_equal 42, first_byte_procmacro("\x2a".b)
  end
end
