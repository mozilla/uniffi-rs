/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Manages source files used during parsing
//!
//! Each source file is read once, then interned using a static hash map.
//! This makes it cheap and easy to pass around file IDs
//! that can be used with `codespan-reporting` for nice error messages.

use std::{
    fs,
    sync::{Mutex, OnceLock},
};

use camino::Utf8Path;
use codespan_reporting::files::SimpleFiles;

use crate::{Error, ErrorKind::*, Result};

#[derive(Clone, Copy, Debug)]
pub struct FileId(pub usize);

impl FileId {
    #[cfg(test)]
    pub fn fake() -> Self {
        Self(0)
    }
}

pub type Files = SimpleFiles<&'static str, &'static str>;

pub fn read_to_string(path: &Utf8Path) -> Result<(FileId, &'static str)> {
    let contents = fs::read_to_string(path)
        .map_err(|e| Error::new_without_location(ReadError(path.to_path_buf(), e.to_string())))?;

    // Leak both the contents and path string so that we can use static strings
    // This means we can never release the memory used to store the file contents,
    // which is okay because no reason to do so.
    let path = path.to_string().leak();
    let contents: &'static str = contents.leak();
    Ok(with_files(|files| {
        let file_id = files.add(path, contents);
        (FileId(file_id), contents)
    }))
}

pub fn with_files<F, T>(f: F) -> T
where
    F: FnOnce(&mut Files) -> T,
{
    static FILES: OnceLock<Mutex<Files>> = OnceLock::new();
    let mut files = FILES
        .get_or_init(|| Mutex::new(Files::new()))
        .lock()
        .unwrap();
    f(&mut files)
}
