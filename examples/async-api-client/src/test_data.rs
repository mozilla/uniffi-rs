/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Sample data downloaded from a real github api call
///
/// The tests don't make real HTTP calls to avoid them failing because of network errors.
pub fn test_response_data() -> String {
    String::from(
        r#"{
  "url": "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017",
  "repository_url": "https://api.github.com/repos/mozilla/uniffi-rs",
  "labels_url": "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017/labels{/name}",
  "comments_url": "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017/comments",
  "events_url": "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017/events",
  "html_url": "https://github.com/mozilla/uniffi-rs/issues/2017",
  "id": 2174982360,
  "node_id": "I_kwDOECpYAM6Bo5jY",
  "number": 2017,
  "title": "Foreign-implemented async traits",
  "user": {
    "login": "bendk",
    "id": 1012809,
    "node_id": "MDQ6VXNlcjEwMTI4MDk=",
    "avatar_url": "https://avatars.githubusercontent.com/u/1012809?v=4",
    "gravatar_id": "",
    "url": "https://api.github.com/users/bendk",
    "html_url": "https://github.com/bendk",
    "followers_url": "https://api.github.com/users/bendk/followers",
    "following_url": "https://api.github.com/users/bendk/following{/other_user}",
    "gists_url": "https://api.github.com/users/bendk/gists{/gist_id}",
    "starred_url": "https://api.github.com/users/bendk/starred{/owner}{/repo}",
    "subscriptions_url": "https://api.github.com/users/bendk/subscriptions",
    "organizations_url": "https://api.github.com/users/bendk/orgs",
    "repos_url": "https://api.github.com/users/bendk/repos",
    "events_url": "https://api.github.com/users/bendk/events{/privacy}",
    "received_events_url": "https://api.github.com/users/bendk/received_events",
    "type": "User",
    "site_admin": false
  },
  "labels": [

  ],
  "state": "open",
  "locked": false,
  "assignee": null,
  "assignees": [

  ],
  "milestone": null,
  "comments": 0,
  "created_at": "2024-03-07T23:07:29Z",
  "updated_at": "2024-03-07T23:07:29Z",
  "closed_at": null,
  "author_association": "CONTRIBUTOR",
  "active_lock_reason": null,
  "body": "We currently allow Rust code to implement async trait methods, but foreign implementations are not supported.  We should extend support to allow for foreign code.\\r\\n\\r\\nI think this is a key feature for full async support.  It allows Rust code to define an async method that depends on a foreign async method.  This allows users to use async code without running a Rust async runtime, you can effectively piggyback on the foreign async runtime.",
  "closed_by": null,
  "reactions": {
    "url": "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017/reactions",
    "total_count": 0,
    "+1": 0,
    "-1": 0,
    "laugh": 0,
    "hooray": 0,
    "confused": 0,
    "heart": 0,
    "rocket": 0,
    "eyes": 0
  },
  "timeline_url": "https://api.github.com/repos/mozilla/uniffi-rs/issues/2017/timeline",
  "performed_via_github_app": null,
  "state_reason": null
}"#,
    )
}
