import unittest
import regression_logging_callback_interface

class Logger:
    def __init__(self):
        self.messages = []

    def log_message(self, message):
        self.messages.append(message)

class TestLoggingCallbackInterface(unittest.TestCase):
    def test_log(self):
        logger = Logger()
        regression_logging_callback_interface.install_logger(logger)
        self.assertEqual(logger.messages, [])
        regression_logging_callback_interface.log_something()
        self.assertIn("something", logger.messages)

if __name__=='__main__':
    unittest.main()
