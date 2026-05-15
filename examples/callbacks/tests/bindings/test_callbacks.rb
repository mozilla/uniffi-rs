# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'callbacks'

# This is defined in UDL as a "callback". It's not possible to have a Rust
# implementation of a callback, they only exist on the foreign side.
class CallAnswererImpl < Callbacks::CallAnswerer
  def initialize(mode)
    @mode = mode
  end

  def answer
    if @mode == 'ready'
      'Bonjour'
    elsif @mode == 'busy'
      raise Callbacks::TelephoneError::Busy
    else
      raise ValueError, 'Testing an unexpected error'
    end
  end
end

# This is a normal Rust trait - very much like a callback but can be implemented
# in Rust or in foreign code and is generally more consistent with the uniffi
# Arc<>-based object model.
class DiscountSim < Callbacks::SimCard
  def name
    'ruby'
  end
end

class TestCallbacks < Test::Unit::TestCase
  TelephoneImpl = Callbacks::Telephone

  def test_answer
    cb_object = CallAnswererImpl.new 'ready'

    assert_equal 'Bonjour', telephone.call(sim, cb_object)
  end

  def test_busy
    cb_object = CallAnswererImpl.new 'busy'

    assert_raise(Callbacks::TelephoneError::Busy) do
      telephone.call sim, cb_object
    end
  end

  def test_unexpected_error
    cb_object = CallAnswererImpl.new 'something-else'

    assert_raise(Callbacks::TelephoneError::InternalTelephoneError) do
      telephone.call sim, cb_object
    end
  end

  def test_sims
    cb_object = CallAnswererImpl.new 'ready'
    sim = DiscountSim.new

    assert_equal 'ruby est bon marché', telephone.call(sim, cb_object)
  end

  def sim
    Callbacks.get_sim_cards[0]
  end

  def telephone
    self.class::TelephoneImpl.new
  end
end

class FancyTestCallbacks < TestCallbacks
  TelephoneImpl = Callbacks::FancyTelephone

  def test_answer
    super
  end

  def test_busy
    super
  end

  def test_unexpected_error
    super
  end

  def test_sims
    super
  end
end
