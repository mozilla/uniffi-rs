# frozen_string_literal: true

require 'test/unit'
require 'documentation'

include Test::Unit::Assertions

assert_equal Documentation.hello(Documentation::Pet.new("Tom")), "Hello Tom!"
assert_equal Documentation::Person.new("Daniel").get_name(), "Daniel"
assert_equal Documentation.add(2, 4), 6
