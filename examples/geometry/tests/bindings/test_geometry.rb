# frozen_string_literal: true

require 'geometry'

include Geometry

def assert(condition)
  raise 'Assertion failed!' unless condition
end

ln1 = Line.new(Point.new(0.0,0.0), Point.new(1.0,2.0))
ln2 = Line.new(Point.new(1.0,1.0), Point.new(2.0,2.0))

assert Geometry.gradient(ln1) == 2
assert Geometry.gradient(ln2) == 1

assert Geometry.intersection(ln1, ln2) == Point.new(0, 0)
assert Geometry.intersection(ln1, ln1).nil?
