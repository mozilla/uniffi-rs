/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Exclude CI items based on the TOML configuration.

use std::collections::HashSet;

use crate::interface::{AsType, ComponentInterface, Constructor, Function, Method, TypeUniverse};

pub fn apply_exclusions(ci: &mut ComponentInterface, exclusions: &[String]) {
    let exclusions: HashSet<&str> = exclusions.iter().map(String::as_str).collect();

    remove_functions(&exclusions, &mut ci.functions);
    remove_type_definitions(&exclusions, &mut ci.records, &mut ci.types);
    remove_type_definitions(&exclusions, &mut ci.enums, &mut ci.types);
    remove_type_definitions(&exclusions, &mut ci.objects, &mut ci.types);
    remove_type_definitions(&exclusions, &mut ci.callback_interfaces, &mut ci.types);

    for o in ci.objects.iter_mut() {
        remove_constructors(&exclusions, &o.name, &mut o.constructors);
        remove_methods(&exclusions, &o.name, &mut o.methods);
    }
    for rec in ci.records.iter_mut() {
        remove_constructors(&exclusions, &rec.name, &mut rec.constructors);
        remove_methods(&exclusions, &rec.name, &mut rec.methods);
    }
    for en in ci.enums.iter_mut() {
        remove_constructors(&exclusions, &en.name, &mut en.constructors);
        remove_methods(&exclusions, &en.name, &mut en.methods);
    }
    for cbi in ci.callback_interfaces.iter_mut() {
        remove_methods(&exclusions, &cbi.name, &mut cbi.methods);
    }
}

fn remove_functions(excludes: &HashSet<&str>, functions: &mut Vec<Function>) {
    functions.retain(|f| !excludes.contains(f.name.as_str()))
}

fn remove_type_definitions<T: AsType>(
    excludes: &HashSet<&str>,
    items: &mut Vec<T>,
    types: &mut TypeUniverse,
) {
    let mut removed_type_names = HashSet::new();
    items.retain(|ty| {
        let ty = ty.as_type();
        let Some(name) = ty.name() else {
            return true;
        };

        if excludes.contains(name) {
            removed_type_names.insert(name.to_string());
            false
        } else {
            true
        }
    });
    types
        .type_definitions
        .retain(|name, _| !removed_type_names.contains(name.as_str()));
    types.all_known_types.retain(|ty| match ty.name() {
        None => true,
        Some(name) => !removed_type_names.contains(name),
    });
}

fn remove_constructors(
    excludes: &HashSet<&str>,
    type_name: &str,
    constructors: &mut Vec<Constructor>,
) {
    constructors.retain(|cons| !excludes.contains(format!("{type_name}.{}", cons.name).as_str()));
}

fn remove_methods(excludes: &HashSet<&str>, type_name: &str, methods: &mut Vec<Method>) {
    methods.retain(|meth| !excludes.contains(format!("{type_name}.{}", meth.name).as_str()));
}
