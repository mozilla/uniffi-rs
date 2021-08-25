/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal};
use askama::Template;
use paste::paste;
use std::fmt;

#[allow(unused_imports)]
use super::filters;

macro_rules! impl_code_type_for_miscellany {
     ($T:ty, $class_name:literal, $canonical_name:literal, $imports:expr, $helper_code:literal) => {
         paste! {
             #[derive(Template)]
             #[template(syntax = "swift", ext = "swift", escape = "none", source = $helper_code )]
             pub struct $T;

             impl CodeType for $T  {
                 fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                     $class_name.into()
                 }

                 fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
                     unreachable!()
                 }

                 fn lift(&self, _oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                     format!("{}.lift({})", $class_name, nm)
                 }

                 fn read(&self, _oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                     format!("{}.read(from: {})", $class_name, nm)
                 }

                 fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                     format!("{}.lower()", oracle.var_name(nm))
                 }

                 fn write(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> String {
                     format!("{}.write(into: {})", oracle.var_name(nm), target)
                 }

                 fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                     Some(self.render().unwrap())
                 }

                 fn import_code(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
                    let imports: Vec<&str> = $imports;
                    if (!imports.is_empty()) {
                        Some(
                            imports.into_iter().map(|s| s.into()).collect()
                        )
                    } else {
                        None
                    }
                 }
             }
         }
     }
 }

impl_code_type_for_miscellany!(
    TimestampCodeType,
    "Date",
    "Timestamp",
    vec![],
    r#"
    extension Date: ViaFfiUsingByteBuffer, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> Self {
            let seconds: Int64 = try buf.readInt()
            let nanoseconds: UInt32 = try buf.readInt()
            if seconds >= 0 {
                let delta = Double(seconds) + (Double(nanoseconds) / 1.0e9)
                return Date.init(timeIntervalSince1970: delta)
            } else {
                let delta = Double(seconds) - (Double(nanoseconds) / 1.0e9)
                return Date.init(timeIntervalSince1970: delta)
            }
        }

        fileprivate func write(into buf: Writer) {
            var delta = self.timeIntervalSince1970
            var sign: Int64 = 1
            if delta < 0 {
                // The nanoseconds portion of the epoch offset must always be
                // positive, to simplify the calculation we will use the absolute
                // value of the offset.
                sign = -1
                delta = -delta
            }
            if delta.rounded(.down) > Double(Int64.max) {
                fatalError("Timestamp overflow, exceeds max bounds supported by Uniffi")
            }
            let seconds = Int64(delta)
            let nanoseconds = UInt32((delta - Double(seconds)) * 1.0e9)
            buf.writeInt(sign * seconds)
            buf.writeInt(nanoseconds)
        }
    }
 "#
);

impl_code_type_for_miscellany!(
    DurationCodeType,
    "TimeInterval",
    "Duration",
    vec![],
    r#"
    extension TimeInterval: ViaFfiUsingByteBuffer, ViaFfi {
        fileprivate static func read(from buf: Reader) throws -> Self {
            let seconds: UInt64 = try buf.readInt()
            let nanoseconds: UInt32 = try buf.readInt()
            return Double(seconds) + (Double(nanoseconds) / 1.0e9)
        }

        fileprivate func write(into buf: Writer) {
            if self.rounded(.down) > Double(Int64.max) {
                fatalError("Duration overflow, exceeds max bounds supported by Uniffi")
            }

            if self < 0 {
                fatalError("Invalid duration, must be non-negative")
            }

            let seconds = UInt64(self)
            let nanoseconds = UInt32((self - Double(seconds)) * 1.0e9)
            buf.writeInt(seconds)
            buf.writeInt(nanoseconds)
        }
    }
"#
);
