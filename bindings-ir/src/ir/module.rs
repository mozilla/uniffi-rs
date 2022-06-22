/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::definitions::*;
use super::helpers::*;
use super::names::*;
use super::statement::*;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, BTreeSet, HashMap};

/// Foreign bindings module
#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct Module {
    pub native_library: Option<NativeLibrary>,
    pub definitions: HashMap<String, Definition>,
    pub tests: Vec<FunctionDef>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_native_library(
        &mut self,
        name: impl Into<String>,
        functions: impl IntoIterator<Item = FFIFunctionDef>,
    ) {
        if self.native_library.is_some() {
            panic!("NativeLibrary already set");
        }
        self.native_library
            .replace(NativeLibrary::new(name, functions));
    }

    fn _add_user_def(&mut self, def: Definition) -> &Definition {
        match self.definitions.entry(def.name().to_string()) {
            Entry::Occupied(inner) => panic!("{} already in use by {:?}", def.name(), inner),
            Entry::Vacant(e) => e.insert(def),
        }
    }

    pub fn add_buffer_stream_class(&mut self, name: impl Into<String>) {
        self._add_user_def(Definition::BufferStream(BufferStreamDef {
            name: ClassName::new(name.into()),
        }));
    }

    pub fn add_cstruct(&mut self, cstruct: CStructDef) {
        self._add_user_def(Definition::CStruct(cstruct));
    }

    pub fn add_function(&mut self, func: FunctionDef) {
        self._add_user_def(Definition::Function(func));
    }

    pub fn add_class(&mut self, class: ClassDef) {
        self._add_user_def(Definition::Class(class));
    }

    pub fn add_data_class(&mut self, data_class: DataClassDef) {
        self._add_user_def(Definition::DataClass(data_class));
    }

    pub fn add_enum(&mut self, enum_: EnumDef) {
        self._add_user_def(Definition::Enum(enum_));
    }

    pub fn add_exception_base(&mut self, mut exc_base: ExceptionBaseDef) {
        match &exc_base.parent {
            None => exc_base.depth = 0,
            Some(parent_name) => {
                let parent = self
                    .get_exception_base(parent_name)
                    .expect("ExceptionBase {parent_name} not found");
                exc_base.depth = parent.depth + 1
            }
        }
        self._add_user_def(Definition::ExceptionBase(exc_base));
    }

    pub fn add_exception(&mut self, exc: ExceptionDef) {
        self._add_user_def(Definition::Exception(exc));
    }

    pub fn get_cstruct(&self, name: &str) -> Option<&CStructDef> {
        self.get_def(name).and_then(Definition::as_cstruct)
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.get_def(name).and_then(Definition::as_function)
    }

    pub fn get_class(&self, name: &str) -> Option<&ClassDef> {
        self.get_def(name).and_then(Definition::as_class)
    }

    pub fn get_enum(&self, name: &str) -> Option<&EnumDef> {
        self.get_def(name).and_then(Definition::as_enum)
    }

    pub fn get_exception_base(&self, name: &str) -> Option<&ExceptionBaseDef> {
        self.get_def(name).and_then(Definition::as_exception_base)
    }

    pub fn get_exception(&self, name: &str) -> Option<&ExceptionDef> {
        self.get_def(name).and_then(Definition::as_exception)
    }

    pub fn get_def(&self, name: &str) -> Option<&Definition> {
        self.definitions.get(name)
    }

    pub fn iter_definitions(&self) -> impl Iterator<Item = &Definition> {
        self.definitions
            .values()
            .into_iter()
            // Collect into a BTreeSet to order the definitions.  This groups each kind together.
            .collect::<BTreeSet<_>>()
            .into_iter()
    }

    pub fn add_test(&mut self, name: impl Into<String>, body: impl IntoIterator<Item = Statement>) {
        self.tests.push(FunctionDef {
            vis: private(),
            name: name.into().into(),
            throws: None,
            args: vec![],
            return_type: None,
            body: block(body),
        })
    }
}
