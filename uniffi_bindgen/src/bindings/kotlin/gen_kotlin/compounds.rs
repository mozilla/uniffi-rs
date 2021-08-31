/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal, TypeIdentifier};
use askama::Template;
use paste::paste;

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

    {% call kt::unsigned_types_annotation(self) %}
    object {{ outer_type|ffi_converter_name }}: FFIConverterRustBuffer<{{ inner_type_name }}?> {
        override fun write(v: {{ inner_type_name }}?, bufferWrite: BufferWriteFunc) {
            if (v == null) {
                putByte(0, bufferWrite)
            } else {
                putByte(1, bufferWrite)
                {{ inner_type|ffi_converter_name }}.write(v, bufferWrite)
            }
        }

        override fun read(buf: ByteBuffer): {{ inner_type_name }}? {
            if (buf.get().toInt() == 0) {
                return null
            }
            return {{ inner_type|ffi_converter_name }}.read(buf)
        }
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

    {% call kt::unsigned_types_annotation(self) %}
    object {{ outer_type|ffi_converter_name }}: FFIConverterRustBuffer<List<{{ inner_type_name }}>> {
        override fun write(v: List<{{ inner_type_name }}>, bufferWrite: BufferWriteFunc) {
            putInt(v.size, bufferWrite)
            v.forEach {
                {{ inner_type|ffi_converter_name }}.write(it, bufferWrite)
            }
        }

        {% call kt::unsigned_types_annotation(self) %}
        override fun read(buf: ByteBuffer): List<{{ inner_type_name }}> {
            val len = buf.getInt()
            return List<{{ inner_type_name }}>(len) {
                {{ inner_type|ffi_converter_name }}.read(buf)
            }
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

    {% call kt::unsigned_types_annotation(self) %}
    object {{ outer_type|ffi_converter_name }}: FFIConverterRustBuffer<Map<String, {{ inner_type_name }}>> {
        override fun write(v: Map<String, {{ inner_type_name }}>, bufferWrite: BufferWriteFunc) {
            putInt(v.size, bufferWrite)
            // The parens on `(k, v)` here ensure we're calling the right method,
            // which is important for compatibility with older android devices.
            // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
            v.forEach { (k, v) ->
                FFIConverterString.write(k, bufferWrite)
                {{ inner_type|ffi_converter_name }}.write(v, bufferWrite)
            }
        }

        override fun read(buf: ByteBuffer): Map<String, {{ inner_type_name }}> {
            // TODO: Once Kotlin's `buildMap` API is stabilized we should use it here.
            val items : MutableMap<String, {{ inner_type_name }}> = mutableMapOf()
            val len = buf.getInt()
            repeat(len) {
                val k = FFIConverterString.read(buf)
                val v = {{ inner_type|ffi_converter_name }}.read(buf)
                items[k] = v
            }
            return items
        }
    }
"#
);
