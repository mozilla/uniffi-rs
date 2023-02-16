import unittest
import documentation

class TestHello(unittest.TestCase):
    def test_hello(self):
        self.assertEqual(documentation.hello(documentation.Pet("Tom")),
                         "Hello Tom!", "Should be `Hello Tom!`")

class TestGetName(unittest.TestCase):
    def test_get_name(self):
        self.assertEqual(documentation.Person("Daniel").get_name(), "Daniel", "Should be Daniel")
        
class TestAdd(unittest.TestCase):
    def test_add(self):
        self.assertEqual(documentation.add(2, 4), 6, "Should be 6")


if __name__ == '__main__':
    unittest.main()
