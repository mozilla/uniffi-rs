/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal, TypeIdentifier};
use askama::Template;
use paste::paste;
use std::fmt;

#[allow(unused_imports)]
use super::filters;

fn render_literal(oracle: &dyn CodeOracle, literal: &Literal, inner: &TypeIdentifier) -> String {
    match literal {
        Literal::Null => "null".into(),
        Literal::EmptySequence => "listOf()".into(),
        Literal::EmptyMap => "mapOf".into(),

        // For optionals
        _ => oracle.find(inner).literal(oracle, literal),
    }
}

macro_rules! impl_code_type_for_compound {
     ($T:ty, $type_label_pattern:literal, $canonical_name_pattern: literal, $helper_code:literal) => {
        paste! {
            #[derive(Template)]
            #[template(syntax = "kt", ext = "kt", escape = "none", source = $helper_code )]
            pub struct $T {
                inner: TypeIdentifier,
                outer: TypeIdentifier,
            }

            impl $T {
                pub fn new(inner: TypeIdentifier, outer: TypeIdentifier) -> Self {
                    Self { inner, outer }
                }
                fn inner(&self) -> &TypeIdentifier {
                    &self.inner
                }
                fn outer(&self) -> &TypeIdentifier {
                    &self.outer
                }
                fn contains_unsigned_types(&self) -> bool {
                    // XXX We're unable to lookup the concrete interface::type from the self.innner type
                    // because we don't have access to the ComponentInterface.
                    // So we err on the side of caution and say true. This is reflected in
                    // ComponentInterface.type_contains_unsigned_types(typ: &Type).
                    true
                }
            }

            impl CodeType for $T  {
                fn type_label(&self, oracle: &dyn CodeOracle) -> String {
                    format!($type_label_pattern, oracle.find(self.inner()).type_label(oracle))
                }

                fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
                    format!($canonical_name_pattern, oracle.find(self.inner()).canonical_name(oracle))
                }

                fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
                    render_literal(oracle, &literal, self.inner())
                }

                fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("lower{}({})", self.canonical_name(oracle), oracle.var_name(nm))
                }

                fn write(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> String {
                    format!("write{}({}, {})", self.canonical_name(oracle), oracle.var_name(nm), target)
                }

                fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("lift{}({})", self.canonical_name(oracle), nm)
                }

                fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("read{}({})", self.canonical_name(oracle), nm)
                }

                fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                    Some(self.render().unwrap())
                }
            }
        }
    }
 }

impl_code_type_for_compound!(
    OptionalCodeType,
    "{}?",
    "Optional{}",
    r#"
    {%- import "macros.kt" as kt -%}
    {%- let inner_type = self.inner() %}
    {%- let outer_type = self.outer() %}
    {%- let inner_type_name = inner_type|type_kt %}
    {%- let canonical_type_name = outer_type|canonical_name %}

    // Helper functions for passing values of type {{ outer_type|type_kt }}
    {% call kt::unsigned_types_annotation(self) %}
    internal fun lower{{ canonical_type_name }}(v: {{ inner_type_name }}?): RustBuffer.ByValue {
        return lowerIntoRustBuffer(v) { v, buf ->
            write{{ canonical_type_name }}(v, buf)
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun write{{ canonical_type_name }}(v: {{ inner_type_name }}?, buf: RustBufferBuilder) {
        if (v == null) {
            buf.putByte(0)
        } else {
            buf.putByte(1)
            {{ "v"|write_kt("buf", inner_type) }}
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): {{ inner_type_name }}? {
        return liftFromRustBuffer(rbuf) { buf ->
            read{{ canonical_type_name }}(buf)
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun read{{ canonical_type_name }}(buf: ByteBuffer): {{ inner_type_name }}? {
        if (buf.get().toInt() == 0) {
            return null
        }
        return {{ "buf"|read_kt(inner_type) }}
    }
 "#
);

impl_code_type_for_compound!(
    SequenceCodeType,
    "List<{}>",
    "Sequence{}",
    r#"
    {%- import "macros.kt" as kt -%}
    {%- let inner_type = self.inner() %}
    {%- let outer_type = self.outer() %}
    {%- let inner_type_name = inner_type|type_kt %}
    {%- let canonical_type_name = outer_type|canonical_name %}

    // Helper functions for passing values of type {{ outer_type|type_kt }}
    {% call kt::unsigned_types_annotation(self) %}
    internal fun lower{{ canonical_type_name }}(v: List<{{ inner_type_name }}>): RustBuffer.ByValue {
        return lowerIntoRustBuffer(v) { v, buf ->
            write{{ canonical_type_name }}(v, buf)
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun write{{ canonical_type_name }}(v: List<{{ inner_type_name }}>, buf: RustBufferBuilder) {
        buf.putInt(v.size)
        v.forEach {
            {{ "it"|write_kt("buf", inner_type) }}
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): List<{{ inner_type_name }}> {
        return liftFromRustBuffer(rbuf) { buf ->
            read{{ canonical_type_name }}(buf)
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun read{{ canonical_type_name }}(buf: ByteBuffer): List<{{ inner_type_name }}> {
        val len = buf.getInt()
        return List<{{ inner_type_name }}>(len) {
            {{ "buf"|read_kt(inner_type) }}
        }
    }
"#
);

impl_code_type_for_compound!(
    MapCodeType,
    "Map<String, {}>",
    "Map{}",
    r#"
    {%- import "macros.kt" as kt -%}
    {%- let inner_type = self.inner() %}
    {%- let outer_type = self.outer() %}
    {%- let inner_type_name = inner_type|type_kt %}
    {%- let canonical_type_name = outer_type|canonical_name %}

    // Helper functions for passing values of type {{ outer_type|type_kt }}
    {% call kt::unsigned_types_annotation(self) %}
    internal fun lower{{ canonical_type_name }}(m: Map<String, {{ inner_type_name }}>): RustBuffer.ByValue {
        return lowerIntoRustBuffer(m) { m, buf ->
            write{{ canonical_type_name }}(m, buf)
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun write{{ canonical_type_name }}(v: Map<String, {{ inner_type_name }}>, buf: RustBufferBuilder) {
        buf.putInt(v.size)
        // The parens on `(k, v)` here ensure we're calling the right method,
        // which is important for compatibility with older android devices.
        // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
        v.forEach { (k, v) ->
            {{ "k"|write_kt("buf", TypeIdentifier::String) }}
            {{ "v"|write_kt("buf", inner_type) }}
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): Map<String, {{ inner_type_name }}> {
        return liftFromRustBuffer(rbuf) { buf ->
            read{{ canonical_type_name }}(buf)
        }
    }

    {% call kt::unsigned_types_annotation(self) %}
    internal fun read{{ canonical_type_name }}(buf: ByteBuffer): Map<String, {{ inner_type_name }}> {
        // TODO: Once Kotlin's `buildMap` API is stabilized we should use it here.
        val items : MutableMap<String, {{ inner_type_name }}> = mutableMapOf()
        val len = buf.getInt()
        repeat(len) {
            val k = {{ "buf"|read_kt(TypeIdentifier::String) }}
            val v = {{ "buf"|read_kt(inner_type) }}
            items[k] = v
        }
        return items
    }
"#
);
