# frozen_string_literal: true

require 'test/unit'
require 'async_api_client'

class RbHttpClient
  def fetch(url, credentials)
    raise AsyncApiClient::ApiError::Http.new('Unauthorized') unless credentials == 'username:password'

    # Fake HTTP response
    ::Kernel.sleep(0.01)
    if url == 'https://api.github.com/repos/mozilla/uniffi-rs/issues/2017'
      AsyncApiClient.test_response_data
    else
      raise AsyncApiClient::ApiError::Http.new("Wrong URL: #{url}")
    end
  end
end

class RbTaskRunner
  def run_task(task)
    task.execute
  end
end

class TestAsyncApiClient < Test::Unit::TestCase
  def test_api_client
    client = AsyncApiClient::ApiClient.new(RbHttpClient.new, RbTaskRunner.new)
    issue = client.get_issue('mozilla', 'uniffi-rs', 2017)

    assert_equal 'Foreign-implemented async traits', issue.title
  end
end
