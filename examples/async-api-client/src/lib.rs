/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod api_client;
mod tasks;
mod test_data;

pub use api_client::{ApiClient, HttpClient, Issue, IssueState};
pub use tasks::{run_task, RustTask, TaskRunner};
pub use test_data::test_response_data;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HttpError: {reason}")]
    Http { reason: String },
    #[error("ApiError: {reason}")]
    Api { reason: String },
    #[error("JsonError: {reason}")]
    Json { reason: String },
}

pub type Result<T> = std::result::Result<T, ApiError>;

uniffi::include_scaffolding!("async-api-client");
