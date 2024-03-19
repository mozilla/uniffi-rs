/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation // To get `DispatchGroup`

#if canImport(async_api_client)
    import async_api_client
#endif

class SwiftHttpClient : HttpClient {
    func fetch(url: String, credentials: String) async throws -> String {
        if (credentials != "username:password") {
            throw ApiError.Http(reason: "Unauthorized")
        }

        // In the real-world we would use an async HTTP library and make a real
        // HTTP request, but to keep the dependencies simple and avoid test
        // fragility we just fake it.
        if (url == "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017") {
            return testResponseData()
        } else {
            throw ApiError.Http(reason: "Wrong URL: \(url)")
        }
    }
}

class SwiftTaskRunner : TaskRunner {
    func runTask(task: RustTask) async {
        let swiftTask = Task { task.execute() }
        let _ = await swiftTask.result
    }
}

var counter = DispatchGroup()
counter.enter()
Task {
    let client = ApiClient(httpClient: SwiftHttpClient(), taskRunner: SwiftTaskRunner())
    let issue = try! await client.getIssue(owner: "mozilla", repository: "uniffi-rs", issueNumber: 2017)
    assert(issue.title == "Foreign-implemented async traits")
    counter.leave()
}
counter.wait()
