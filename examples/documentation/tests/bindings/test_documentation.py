import unittest
import documentation

class TestHello(unittest.TestCase):
    def test_hello(self):
        self.assertEqual(documentation.hello(documentation.Person("Tom")),
                         "Hello Tom!", "Should be `Hello Tom!`")


class TestAdd(unittest.TestCase):
    def test_add(self):
        self.assertEqual(documentation.add(2, 3), 5, "Should be 5")


if __name__ == '__main__':
    unittest.main()
