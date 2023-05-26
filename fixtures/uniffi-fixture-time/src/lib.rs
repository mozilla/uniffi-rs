/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::time::{Duration, SystemTime};

use chrono::offset::Utc;
use chrono::DateTime;

#[derive(Debug, thiserror::Error)]
pub enum ChronologicalError {
    #[error("Time overflow on an operation with {a:?} and {b:?}")]
    TimeOverflow { a: SystemTime, b: Duration },
    #[error("Time difference error {a:?} is before {b:?}")]
    TimeDiffError { a: SystemTime, b: SystemTime },
}

fn return_timestamp(a: SystemTime) -> Result<SystemTime> {
    Ok(a)
}

fn return_duration(a: Duration) -> Result<Duration> {
    Ok(a)
}

fn to_string_timestamp(a: SystemTime) -> String {
    let datetime: DateTime<Utc> = a.into();
    datetime.format("%Y-%m-%dT%H:%M:%S.%fZ").to_string()
}

fn get_pre_epoch_timestamp() -> SystemTime {
    std::time::SystemTime::UNIX_EPOCH
        .checked_sub(std::time::Duration::new(1, 1_000_000))
        .unwrap()
}

fn add(a: SystemTime, b: Duration) -> Result<SystemTime> {
    a.checked_add(b)
        .ok_or(ChronologicalError::TimeOverflow { a, b })
}

fn diff(a: SystemTime, b: SystemTime) -> Result<Duration> {
    a.duration_since(b)
        .map_err(|_| ChronologicalError::TimeDiffError { a, b })
}

fn now() -> SystemTime {
    SystemTime::now()
}

fn equal(a: SystemTime, b: SystemTime) -> bool {
    a == b
}

fn optional(a: Option<SystemTime>, b: Option<Duration>) -> bool {
    a.is_some() && b.is_some()
}

fn get_seconds_before_unix_epoch(b: SystemTime) -> Result<u64> {
    diff(SystemTime::UNIX_EPOCH, b).map(|duration| duration.as_secs())
}

fn set_seconds_before_unix_epoch(seconds: u64) -> Result<SystemTime> {
    let a = SystemTime::UNIX_EPOCH;
    let b = Duration::from_secs(seconds);

    a.checked_sub(b)
        .ok_or(ChronologicalError::TimeOverflow { a, b })
}

type Result<T, E = ChronologicalError> = std::result::Result<T, E>;

uniffi::include_scaffolding!("chronological");
