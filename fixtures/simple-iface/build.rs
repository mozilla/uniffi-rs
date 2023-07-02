/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

fn main() {
    // We previously had a UDL with content `namespace uniffi_simple_iface {};`
    uniffi::generate_namespaced_scaffolding("uniffi_simple_iface").unwrap();
}
