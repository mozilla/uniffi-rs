/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import swift_mock_objects

// Swift version 6 gives a warning that we must restate `@unchecked Sendable`, however earlier
// versions of Swift fail if we add that.  Let's leave it out and accept the warning for now.
public class TestObjectMock: TestObject {

    required public init(unsafeFromHandle handle: UInt64) {
        fatalError("Not supported")
    }

    public init() {
        super.init(noHandle: NoHandle.init())
    }

    // Override the "is_mock()" function

    override public func isMock() -> Bool {
        return true;
    }
}

var mocked: TestObjectMock? = TestObjectMock()

// Test that our mock object can override the functions of the real object
assert(mocked!.isMock())

// Test that we don't crash when deinitializing the mock object
mocked = nil
