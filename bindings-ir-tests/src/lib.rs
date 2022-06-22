/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod basic;
mod classes;
mod compounds;
mod control_flow;
mod cstructs;
mod enums;
mod exceptions;
mod ffi;
mod math;
mod pointers;
mod strings;
mod testdir;
mod testlib;

pub use basic::basic_tests;
pub use classes::class_tests;
pub use compounds::compound_tests;
pub use control_flow::control_flow_tests;
pub use cstructs::cstructs_tests;
pub use enums::enum_tests;
pub use exceptions::exception_tests;
pub use ffi::ffi_tests;
pub use math::math_tests;
pub use pointers::pointer_tests;
pub use strings::string_tests;
pub use testdir::setup_test_dir;

use testlib::test_module;
