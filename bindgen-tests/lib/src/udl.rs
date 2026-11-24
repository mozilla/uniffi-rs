/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub struct UdlType {
    pub a: f64,
    pub b: f64,
}

pub fn udl_sum_fields(udl_type: &UdlType) -> f64 {
    udl_type.a + udl_type.b
}

pub fn udl_make_type(a: f64, b: f64) -> UdlType {
    UdlType { a, b }
}

/// Simulates a crate that doesn't use UniFFI.
///
/// The UDL file uses `[Remote]` to describe this type.
pub mod remote {
    pub struct UdlRemoteType {
        pub a: f64,
        pub b: f64,
    }
}

pub use remote::UdlRemoteType;

pub fn udl_remote_sum_fields(remote_type: UdlRemoteType) -> f64 {
    remote_type.a + remote_type.b
}
