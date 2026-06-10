require 'uniffi_example_futures'

result = UniffiExampleFutures.say_after 200, 'Alice'
raise "Expected 'Hello, Alice!', got '#{result}'" unless result == 'Hello, Alice!'
