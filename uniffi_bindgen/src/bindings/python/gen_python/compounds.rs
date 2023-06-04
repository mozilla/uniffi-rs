/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeOracle, CodeType, Literal, TypeIdentifier};

#[derive(Debug)]
pub struct OptionalCodeType {
    inner: TypeIdentifier,
}

impl OptionalCodeType {
    pub fn new(inner: TypeIdentifier) -> Self {
        Self { inner }
    }
}

impl CodeType for OptionalCodeType {
    fn type_label(&self) -> String {
        super::PythonCodeOracle.find(&self.inner).type_label()
    }

    fn canonical_name(&self) -> String {
        format!(
            "Optional{}",
            super::PythonCodeOracle.find(&self.inner).canonical_name(),
        )
    }

    fn literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::Null => "None".into(),
            _ => super::PythonCodeOracle.find(&self.inner).literal(literal),
        }
    }
}

#[derive(Debug)]
pub struct SequenceCodeType {
    inner: TypeIdentifier,
}

impl SequenceCodeType {
    pub fn new(inner: TypeIdentifier) -> Self {
        Self { inner }
    }
}

impl CodeType for SequenceCodeType {
    fn type_label(&self) -> String {
        "list".to_string()
    }

    fn canonical_name(&self) -> String {
        format!(
            "Sequence{}",
            super::PythonCodeOracle.find(&self.inner).canonical_name(),
        )
    }

    fn literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::EmptySequence => "[]".into(),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct MapCodeType {
    key: TypeIdentifier,
    value: TypeIdentifier,
}

impl MapCodeType {
    pub fn new(key: TypeIdentifier, value: TypeIdentifier) -> Self {
        Self { key, value }
    }
}

impl CodeType for MapCodeType {
    fn type_label(&self) -> String {
        "dict".to_string()
    }

    fn canonical_name(&self) -> String {
        format!(
            "Map{}{}",
            super::PythonCodeOracle.find(&self.key).canonical_name(),
            super::PythonCodeOracle.find(&self.value).canonical_name(),
        )
    }

    fn literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::EmptyMap => "{}".into(),
            _ => unimplemented!(),
        }
    }
}
