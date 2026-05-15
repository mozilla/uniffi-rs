# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'traits'

class RbButton < Traits::Button
  def name
    'RbButton'
  end
end

class TestTraits < Test::Unit::TestCase
  def test_first_button
    buttons = Traits.get_buttons

    assert_equal 'stop', buttons[0].name
    assert_equal 'stop', Traits.press(buttons[0]).name
  end

  def test_second_button
    buttons = Traits.get_buttons

    assert_equal 'go', buttons[1].name
    assert_equal 'go', Traits.press(buttons[1]).name
  end

  def test_rb_button
    assert_equal 'RbButton', Traits.press(RbButton.new).name
  end
end
