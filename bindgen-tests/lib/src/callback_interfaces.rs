/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::errors::TestError;

#[uniffi::export(callback_interface)]
pub trait TestCallbackInterface {
    /// No-op function, this tests if that we can make calls at all
    fn noop(&self);
    /// Get the internal value
    fn get_value(&self) -> u32;
    /// Set the internal value
    fn set_value(&self, value: u32);
    /// Method aimed at maximizing the complexity
    ///
    /// This should return an error if `numbers.a == numbers.b` otherwise it should return numbers back
    /// unchanged.
    fn throw_if_equal(
        &self,
        numbers: CallbackInterfaceNumbers,
    ) -> Result<CallbackInterfaceNumbers, TestError>;
}

#[derive(uniffi::Record)]
pub struct CallbackInterfaceNumbers {
    pub a: u32,
    pub b: u32,
}

#[uniffi::export]
fn invoke_test_callback_interface_noop(cbi: Box<dyn TestCallbackInterface>) {
    cbi.noop()
}

#[uniffi::export]
fn invoke_test_callback_interface_get_value(cbi: Box<dyn TestCallbackInterface>) -> u32 {
    cbi.get_value()
}

#[uniffi::export]
fn invoke_test_callback_interface_set_value(cbi: Box<dyn TestCallbackInterface>, value: u32) {
    cbi.set_value(value)
}

#[uniffi::export]
fn invoke_test_callback_interface_throw_if_equal(
    cbi: Box<dyn TestCallbackInterface>,
    numbers: CallbackInterfaceNumbers,
) -> Result<CallbackInterfaceNumbers, TestError> {
    cbi.throw_if_equal(numbers)
}
