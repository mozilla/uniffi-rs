/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub struct UserModel {
    name: String,
}

pub trait ClientDelegate: Sync + Send {
    fn did_receive_friend_request(&self, user: UserModel);
}

#[derive(Debug, Clone)]
struct Client;

impl Client {
    fn new() -> Self {
        Self
    }

    fn friend_request(&self, delegate: Box<dyn ClientDelegate>) {
        let user = UserModel {
            name: "Alice".to_string(),
        };
        delegate.did_receive_friend_request(user);
    }
}

include!(concat!(env!("OUT_DIR"), "/test.uniffi.rs"));
