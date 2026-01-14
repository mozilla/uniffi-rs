/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// `funcWithError` throws when 0 or 1 is passed to it
do {
    try funcWithError(input: 0);
    fatalError("funcWithError should have thrown")
} catch TestError.Failure1 {
    // Expected
}

do {
    try funcWithError(input: 1);
    fatalError("funcWithError should have thrown")
} catch TestError.Failure2 {
    // Expected
}

do {
    try funcWithFlatError(input: 0);
    fatalError("funcWithError should have thrown")
} catch TestFlatError.IoError {
    // Expected
}

// These shouldn't throw
try! funcWithError(input: 2);
try! funcWithFlatError(input: 1);
