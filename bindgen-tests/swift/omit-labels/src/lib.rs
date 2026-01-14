/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

fn no_args() -> i32 {
    31
}

fn one_arg(amount: i32) -> i32 {
    amount
}

fn multiple_args(amount: i32, _msg: String) -> i32 {
    amount
}

uniffi::include_scaffolding!("omit_argument_labels");
