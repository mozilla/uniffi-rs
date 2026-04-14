/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests
import Foundation

let now = Date()
assert(
    abs(roundtripSystemtime(a: now).timeIntervalSinceReferenceDate - now.timeIntervalSinceReferenceDate) < 1
);
assert(roundtripDuration(a: TimeInterval(100)) == TimeInterval(100));
