// Regression test for https://github.com/mozilla/uniffi-rs/issues/1312
//
// Really this can be detected at compile time:
// We didn't apply the same `omit_argument_labels` configuration to callbacks,
// leading to a compilation error.
// To make sure everything gets called right though we write a full test here.

import regression_test_callbacks_omit_labels

final class ClientDelegateImpl: ClientDelegate, @unchecked Sendable {
    var recvCount: Int = 0
    var lastFriend = ""

    func didReceiveFriendRequest(_ user: UserModel) {
        recvCount += 1
        lastFriend = user.name
    }
}

let cd = ClientDelegateImpl()
let client = Client()

client.friendRequest(cd)

assert(cd.recvCount == 1, "delegate should be called once")
assert(cd.lastFriend == "Alice", "delegate should have received Alice's friend request")
