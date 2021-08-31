/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal};
use askama::Template;
use paste::paste;

#[allow(unused_imports)]
use super::filters;

macro_rules! impl_code_type_for_miscellany {
     ($T:ty, $class_name:literal, $canonical_name:literal, $imports:expr, $helper_code:literal) => {
         paste! {
             #[derive(Template)]
             #[template(syntax = "kt", ext = "kt", escape = "none", source = $helper_code )]
             pub struct $T;

             impl CodeType for $T  {
                 fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                     $class_name.into()
                 }

                 fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
                     $canonical_name.into()
                 }

                 fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
                     unreachable!()
                 }

                 fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                     Some(self.render().unwrap())
                 }

                 fn import_code(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
                    Some(
                        $imports.into_iter().map(|s| s.into()).collect()
                    )
                 }
             }
         }
     }
 }

impl_code_type_for_miscellany!(
    TimestampCodeType,
    "java.time.Instant",
    "Timestamp",
    vec!["java.time.Instant", "java.time.DateTimeException"],
    r#"
    object FFIConverterTimestamp: FFIConverter<java.time.Instant, RustBuffer.ByValue> {
         override fun lift(v: RustBuffer.ByValue): java.time.Instant {
             return liftFromRustBuffer(v) { buf ->
                 read(buf)
             }
         }

         override fun read(buf: ByteBuffer): java.time.Instant {
             val seconds = buf.getLong()
             // Type mismatch (should be u32) but we check for overflow/underflow below
             val nanoseconds = buf.getInt().toLong()
             if (nanoseconds < 0) {
                 throw java.time.DateTimeException("Instant nanoseconds exceed minimum or maximum supported by uniffi")
             }
             if (seconds >= 0) {
                 return java.time.Instant.EPOCH.plus(java.time.Duration.ofSeconds(seconds, nanoseconds))
             } else {
                 return java.time.Instant.EPOCH.minus(java.time.Duration.ofSeconds(-seconds, nanoseconds))
             }
         }

         override fun lower(v: java.time.Instant): RustBuffer.ByValue {
             return lowerIntoRustBuffer(v) { v, buf ->
                 write(v, buf)
             }
         }

         override fun write(v: java.time.Instant, bufferWrite: BufferWriteFunc) {
             var epochOffset = java.time.Duration.between(java.time.Instant.EPOCH, v)

             var sign = 1
             if (epochOffset.isNegative()) {
                 sign = -1
                 epochOffset = epochOffset.negated()
             }

             if (epochOffset.nano < 0) {
                 // Java docs provide guarantee that nano will always be positive, so this should be impossible
                 // See: https://docs.oracle.com/javase/8/docs/api/java/time/Instant.html
                 throw IllegalArgumentException("Invalid timestamp, nano value must be non-negative")
             }

             putLong(sign * epochOffset.seconds, bufferWrite)
             // Type mismatch (should be u32) but since values will always be between 0 and 999,999,999 it should be OK
             putInt(epochOffset.nano, bufferWrite)
         }
     }
 "#
);

impl_code_type_for_miscellany!(
    DurationCodeType,
    "java.time.Duration",
    "Duration",
    vec!["java.time.Duration", "java.time.DateTimeException"],
    r#"

    object FFIConverterDuration: FFIConverter<java.time.Duration, RustBuffer.ByValue> {
        override fun lift(v: RustBuffer.ByValue): java.time.Duration {
            return liftFromRustBuffer(v) { buf ->
                read(buf)
            }
        }

        override fun read(buf: ByteBuffer): java.time.Duration {
            // Type mismatch (should be u64) but we check for overflow/underflow below
            val seconds = buf.getLong()
            // Type mismatch (should be u32) but we check for overflow/underflow below
            val nanoseconds = buf.getInt().toLong()
            if (seconds < 0) {
                throw java.time.DateTimeException("Duration exceeds minimum or maximum value supported by uniffi")
            }
            if (nanoseconds < 0) {
                throw java.time.DateTimeException("Duration nanoseconds exceed minimum or maximum supported by uniffi")
            }
            return java.time.Duration.ofSeconds(seconds, nanoseconds)
        }

        override fun lower(v: java.time.Duration): RustBuffer.ByValue {
            return lowerIntoRustBuffer(v) { v, buf ->
                write(v, buf)
            }
        }

        override fun write(v: java.time.Duration, bufferWrite: BufferWriteFunc) {
            if (v.seconds < 0) {
                // Rust does not support negative Durations
                throw IllegalArgumentException("Invalid duration, must be non-negative")
            }

            if (v.nano < 0) {
                // Java docs provide guarantee that nano will always be positive, so this should be impossible
                // See: https://docs.oracle.com/javase/8/docs/api/java/time/Duration.html
                throw IllegalArgumentException("Invalid duration, nano value must be non-negative")
            }

            // Type mismatch (should be u64) but since Rust doesn't support negative durations we should be OK
            putLong(v.seconds, bufferWrite)
            // Type mismatch (should be u32) but since values will always be between 0 and 999,999,999 it should be OK
            putInt(v.nano, bufferWrite)
        }
    }
"#
);
