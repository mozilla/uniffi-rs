# frozen_string_literal: true

require 'arithmetic'

def assert(condition)
  raise 'Assertion failed!' unless condition
end

begin
  Arithmetic.add 18_446_744_073_709_551_615, 1

  raise 'Should have thrown a IntegerOverflow exception!'
rescue Arithmetic::ArithmeticError::IntegerOverflow
  # It's okay!
end

assert Arithmetic.add(2, 4) == 6
assert Arithmetic.add(4, 8) == 12

begin
  Arithmetic.sub 0, 1

  raise 'Should have thrown a IntegerOverflow exception!'
rescue Arithmetic::ArithmeticError::IntegerOverflow
  # It's okay!
end

assert Arithmetic.sub(4, 2) == 2
assert Arithmetic.sub(8, 4) == 4
assert Arithmetic.div(8, 4) == 2

begin
  Arithmetic.div 8, 0

  raise 'Should have thrown a IntegerOverflow exception!'
rescue Arithmetic::InternalError
  # It's okay!
end

assert Arithmetic.equal(2, 2)
assert Arithmetic.equal(4, 4)

assert !Arithmetic.equal(2, 4)
assert !Arithmetic.equal(4, 8)
