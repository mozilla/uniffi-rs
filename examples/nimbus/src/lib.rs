/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone)]
pub struct ApplicationContext {
    app_id: Option<String>,
    app_version: Option<String>,
    locale: Option<String>,
    device_manufacturer: Option<String>,
    device_model: Option<String>,
    region: Option<String>,
    debug_tag: Option<String>,
}

pub struct Experiment {
    slug: String,
    user_facing_name: String,
    user_facing_description: String,
}

pub struct Config {
    server_url: Option<String>,
    uuid: Option<String>,
    collection_name: Option<String>,
    bucket_name: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExperimentError {
    #[error("Error from the network")]
    NetworkError,
    #[error("Error retrieving from the database")]
    DbError,
}

type Result<T, E = ExperimentError> = std::result::Result<T, E>;
pub struct Experiments {}

impl Experiments {
    pub fn new(_: ApplicationContext, _: String, _: Option<Config>) -> Result<Self> {
        unimplemented!()
    }

    pub fn get_experiment_branch(&self, _: String) -> Option<String> {
        unimplemented!()
    }

    pub fn get_active_experiments(&self) -> Vec<Experiment> {
        unimplemented!()
    }

    pub fn opt_in_with_branch(&self, _: String, _: String) {
        unimplemented!()
    }

    pub fn opt_out(&self, _: String) {
        unimplemented!()
    }

    pub fn opt_out_all(&self) {
        unimplemented!()
    }

    pub fn update_experiments(&self) -> Result<()> {
        unimplemented!()
    }
}

include!(concat!(env!("OUT_DIR"), "/nimbus.uniffi.rs"));
