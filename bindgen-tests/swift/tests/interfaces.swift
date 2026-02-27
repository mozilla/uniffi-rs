/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

let interface = TestInterface.init(value: 20);
assert(interface.getValue() == 20);
assert(cloneInterface(interface: interface).getValue() == 20);

// Test records that store interfaces
//
// The goal is to test if we can read/write interface handles to RustBuffers
let two = TwoTestInterfaces(
  first: TestInterface.init(value: 1),
  second: TestInterface.init(value: 2)
);
let swapped = swapTestInterfaces(interfaces: two);
assert(swapped.first.getValue() == 2);
assert(swapped.second.getValue() == 1);

// Create 2 references to an interface using a bunch of intermediary objects:
//   * The one passed to `funcThatClonesInterface`
//   * The clones created for each method call

let interface2 = TestInterface.init(value: 20);
func funcThatClonesInterface(interface: TestInterface) -> TestInterface {
  return cloneInterface(interface: interface);
}
let interface2Clone = funcThatClonesInterface(
    interface: cloneInterface(interface: interface)
);
let _ = interface.getValue();
// Check that only the 2 actual references remain after the dust clears
assert(interface.refCount() == 2);
