/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone)]
pub struct Dictionnaire {
    un: Enumeration,
    deux: bool,
    petit_nombre: u8,
    gros_nombre: u64,
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

fn copie_dictionnaire(d: Dictionnaire) -> Dictionnaire {
    d
}

fn switcheroo(b: bool) -> bool {
    !b
}

include!(concat!(env!("OUT_DIR"), "/rondpoint.uniffi.rs"));
