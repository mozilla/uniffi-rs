/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

pub fn get_dict_1() -> HashMap<String, String> {
    HashMap::new()
}

pub fn get_dict_2() -> HashMap<String, Vec<String>> {
    HashMap::new()
}

pub fn get_dict_3() -> HashMap<String, HashMap<String, String>> {
    HashMap::new()
}

pub fn get_dict_4() -> HashMap<String, HashMap<String, Vec<String>>> {
    HashMap::new()
}

include!(concat!(env!("OUT_DIR"), "/test.uniffi.rs"));
