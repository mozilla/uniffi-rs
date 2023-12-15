/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.async_api_client.*

class KtHttpClient : HttpClient {
    override suspend fun fetch(url: String): String {
        // In the real-world we would use an async HTTP library and make a real
        // HTTP request, but to keep the dependencies simple and avoid test
        // fragility we just fake it.
        if (url == "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017") {
            return testResponseData()
        } else {
            throw ApiException.Http("Wrong URL: ${url}")
        }
    }
}

kotlinx.coroutines.runBlocking {
    val client = ApiClient(KtHttpClient())
    val issue = client.getIssue("mozilla", "uniffi-rs", 2017u)
    assert(issue.title == "Foreign-implemented async traits")
}
