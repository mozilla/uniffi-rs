/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Enum that lists the languages we generate code for

#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Eq, Ord)]
pub enum Language {
    Rust,
    Kotlin,
    Swift,
    Python,
    Ruby,
}

impl std::convert::TryFrom<&str> for Language {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "rust" => Ok(Language::Rust),
            "kotlin" => Ok(Language::Kotlin),
            "swift" => Ok(Language::Swift),
            "python" => Ok(Language::Python),
            "ruby" => Ok(Language::Ruby),
            _ => Err(format!("Unknown language: {}", value))
        }
    }
}
