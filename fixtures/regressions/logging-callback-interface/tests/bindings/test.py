import unittest
import regression_logging_callback_interface

class Logger:
    def __init__(self):
        self.messages = []

    def log_message(self, message, level):
        self.messages.append((message, level))

class TestLoggingCallbackInterface(unittest.TestCase):
    def test_log(self):
        logger = Logger()
        regression_logging_callback_interface.install_logger(logger)
        self.assertEqual(logger.messages, [])
        expected = [
            # Invoke fn `log_foo_at_warn`, UniFFI internals logs its name at Level::TRACE
            ("log_foo_at_warn", "TRACE"), 
            # `log_foo_at_warn`'s body logs "foo" at Level::WARN
            ("foo", "WARN"),
            # Invoke fn `log_bar_at_info`, UniFFI internals logs its name at Level::TRACE
            ("log_bar_at_info", "TRACE"), 
            # `log_bar_at_info`'s body logs "bar" at Level::INFO
            ("bar", "INFO"),
            # Invoke fn `set_log_level_to_debug`, UniFFI internals logs its name at Level::TRACE
            ("set_log_level_to_debug", "TRACE"), 
            # N.B. `("log_buzz_at_error", "TRACE")` is filtered out since max level is DEBUG
            # `log_buzz_at_error`'s body logs "buzz" at Level::ERROR
            ("buzz", "ERROR"),
        ]
        regression_logging_callback_interface.log_foo_at_warn()
        regression_logging_callback_interface.log_bar_at_info()
        regression_logging_callback_interface.set_log_level_to_debug()
        regression_logging_callback_interface.log_buzz_at_error()
        self.assertEqual(logger.messages, expected) 

if __name__=='__main__':
    unittest.main()
