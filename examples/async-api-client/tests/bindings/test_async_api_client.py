import asyncio
import unittest
from urllib.request import urlopen
from async_api_client import *

# Http client that the Rust code depends on
class PyHttpClient(HttpClient):
    async def fetch(self, url):
        # In the real-world we would use something like aiohttp and make a real HTTP request, but to keep
        # the dependencies simple and avoid test fragility we just fake it.
        await asyncio.sleep(0.01)
        if url == "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017":
            return test_response_data()
        else:
            raise ApiError.Http(f"Wrong URL: {url}")

class CallbacksTest(unittest.IsolatedAsyncioTestCase):
    async def test_api_client(self):
        client = ApiClient(PyHttpClient())
        issue = await client.get_issue("mozilla", "uniffi-rs", 2017)
        self.assertEqual(issue.title, "Foreign-implemented async traits")

unittest.main()

