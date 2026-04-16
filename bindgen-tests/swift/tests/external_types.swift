/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

assert(roundtripExtRecord(rec: ExternalRec(a: 42)) == ExternalRec(a: 42))

assert(roundtripExtEnum(en: ExternalEnum.two) == ExternalEnum.two)

let interface = ExternalInterface(value: 20)
assert(roundtripExtInterface(interface: interface).getValue() == 20)

assert(roundtripExtCustomType(custom: 100) == 100);
