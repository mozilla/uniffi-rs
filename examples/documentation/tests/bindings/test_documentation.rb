require "test/unit"
require "documentation"
 
class TestAdd < Test::Unit::TestCase
  def test_hello
    assert_equal(Documentation.hello(Documentation::Person.new("Tom")), "Hello Tom!")
  end

  def test_add
    assert_equal(5, Documentation.add(2, 3))
  end
end
