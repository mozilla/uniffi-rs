# frozen_string_literal: true

require 'test/unit'
require 'remote_types'

class TestRemoteTypes < Test::Unit::TestCase
  include RemoteTypes

  def test_logger
    testLogger = LogSink.new 'SomeFile'
    testLogger.log LogLevel::INFO, 'Hello world'
  end

  def test_error_handling
    assert_raises AnyhowError do
      LogSink.new ''
      raise RuntimeError, 'Constructor should have thrown'
    end
  end
end
