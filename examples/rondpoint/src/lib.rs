/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Dictionnaire {
    un: Enumeration,
    deux: bool,
    petit_nombre: u8,
    gros_nombre: u64,
}

#[derive(Debug, Clone)]
pub struct DictionnaireNombres {
    petit_nombre: u8,
    court_nombre: u16,
    nombre_simple: u32,
    gros_nombre: u64,
}

#[derive(Debug, Clone)]
pub struct DictionnaireNombresSignes {
    petit_nombre: i8,
    court_nombre: i16,
    nombre_simple: i32,
    gros_nombre: i64,
}

#[derive(Debug, Clone)]
pub enum Enumeration {
    Un,
    Deux,
    Trois,
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct minusculeMAJUSCULEDict {
    minusculeMAJUSCULEField: bool,
}

#[allow(non_camel_case_types)]
enum minusculeMAJUSCULEEnum {
    minusculeMAJUSCULEVariant,
}

fn copie_enumeration(e: Enumeration) -> Enumeration {
    e
}

fn copie_enumerations(e: Vec<Enumeration>) -> Vec<Enumeration> {
    e
}

fn copie_carte(e: HashMap<String, Enumeration>) -> HashMap<String, Enumeration> {
    e
}

fn copie_dictionnaire(d: Dictionnaire) -> Dictionnaire {
    d
}

fn switcheroo(b: bool) -> bool {
    !b
}

// Test that values can traverse both ways across the FFI.
// Even if roundtripping works, it's possible we have
// symmetrical errors that cancel each other out.
#[derive(Debug, Clone)]
struct Retourneur;
impl Retourneur {
    fn new() -> Self {
        Retourneur
    }
    fn identique_i8(&self, value: i8) -> i8 {
        value
    }
    fn identique_u8(&self, value: u8) -> u8 {
        value
    }
    fn identique_i16(&self, value: i16) -> i16 {
        value
    }
    fn identique_u16(&self, value: u16) -> u16 {
        value
    }
    fn identique_i32(&self, value: i32) -> i32 {
        value
    }
    fn identique_u32(&self, value: u32) -> u32 {
        value
    }
    fn identique_i64(&self, value: i64) -> i64 {
        value
    }
    fn identique_u64(&self, value: u64) -> u64 {
        value
    }
    fn identique_float(&self, value: f32) -> f32 {
        value
    }
    fn identique_double(&self, value: f64) -> f64 {
        value
    }
    fn identique_boolean(&self, value: bool) -> bool {
        value
    }
    fn identique_string(&self, value: String) -> String {
        value
    }
    fn identique_nombres_signes(
        &self,
        value: DictionnaireNombresSignes,
    ) -> DictionnaireNombresSignes {
        value
    }
    fn identique_nombres(&self, value: DictionnaireNombres) -> DictionnaireNombres {
        value
    }
}

#[derive(Debug, Clone)]
struct Stringifier;

#[allow(dead_code)]
impl Stringifier {
    fn new() -> Self {
        Stringifier
    }
    fn to_string_i8(&self, value: i8) -> String {
        value.to_string()
    }
    fn to_string_u8(&self, value: u8) -> String {
        value.to_string()
    }
    fn to_string_i16(&self, value: i16) -> String {
        value.to_string()
    }
    fn to_string_u16(&self, value: u16) -> String {
        value.to_string()
    }
    fn to_string_i32(&self, value: i32) -> String {
        value.to_string()
    }
    fn to_string_u32(&self, value: u32) -> String {
        value.to_string()
    }
    fn to_string_i64(&self, value: i64) -> String {
        value.to_string()
    }
    fn to_string_u64(&self, value: u64) -> String {
        value.to_string()
    }
    fn to_string_float(&self, value: f32) -> String {
        value.to_string()
    }
    fn to_string_double(&self, value: f64) -> String {
        value.to_string()
    }
    fn to_string_boolean(&self, value: bool) -> String {
        value.to_string()
    }
    fn well_known_string(&self, value: String) -> String {
        format!("uniffi 💚 {}!", value)
    }
}

include!(concat!(env!("OUT_DIR"), "/rondpoint.uniffi.rs"));
