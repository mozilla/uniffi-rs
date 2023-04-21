/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::{Once, RwLock};

// Logger trait that the foreign code implements
pub trait Logger: Sync + Send {
    fn log_message(&self, message: String);
}

// Logger struct that implements the `log::Log` trait.
struct RustLogger(RwLock<Option<Box<dyn Logger>>>);

static RUST_LOGGER: RustLogger = RustLogger(RwLock::new(None));

impl log::Log for RustLogger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        if let Some(foreign_logger) = &*self.0.read().unwrap() {
            foreign_logger.log_message(record.args().to_string());
        }
    }

    fn flush(&self) {}
}

fn init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        log::set_logger(&RUST_LOGGER).expect("Error in install_logger()");
        log::set_max_level(log::LevelFilter::Debug);
    });
}

pub fn install_logger(logger: Box<dyn Logger>) {
    init();
    *RUST_LOGGER.0.write().unwrap() = Some(logger);
}

pub fn log_something() {
    log::warn!("something");
}

uniffi::include_scaffolding!("test");
