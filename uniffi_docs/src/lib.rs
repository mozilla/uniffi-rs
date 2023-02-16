/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, fs::read_to_string, str::FromStr};

use anyhow::Result;
use camino::Utf8Path;
use pulldown_cmark::{Event, HeadingLevel::H1, Parser, Tag};
use syn::Attribute;

/// Function documentation.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub description: String,
    pub arguments_descriptions: HashMap<String, String>,
    pub return_description: Option<String>,
}

impl FromStr for Function {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut description_buff = String::new();
        let mut args_values_buff: Vec<String> = Vec::new();
        let mut args_keys_buff: Vec<String> = Vec::new();

        let mut return_description_buff = String::new();

        let mut current_stage = ParseStage::Description;

        let parser = Parser::new(s);

        for event in parser {
            match event {
                Event::Start(Tag::Heading(H1, _, _)) => match current_stage {
                    ParseStage::Description => current_stage = ParseStage::Arguments,
                    ParseStage::Arguments => current_stage = ParseStage::ReturnDescription,
                    ParseStage::ReturnDescription => (),
                },
                Event::Text(s) => match current_stage {
                    ParseStage::Description => {
                        description_buff.push_str(&s);
                        description_buff.push('\n');
                    }
                    ParseStage::Arguments => {
                        if s.to_lowercase() == "arguments" {
                            continue;
                        }
                        args_values_buff.push(s.to_string());
                    }
                    ParseStage::ReturnDescription => {
                        if s.to_lowercase() == "returns" {
                            continue;
                        }
                        return_description_buff.push_str(&s);
                        return_description_buff.push('\n');
                    }
                },
                Event::Code(s) => {
                    args_keys_buff.push(s.to_string());
                }
                _ => (),
            }
        }

        let mut arguments_descriptions = HashMap::with_capacity(args_keys_buff.len());
        args_keys_buff
            .into_iter()
            .zip(args_values_buff.into_iter())
            .for_each(|(k, v)| {
                arguments_descriptions.insert(k, v.replace('-', "").trim().to_string());
            });

        let return_description = if return_description_buff.is_empty() {
            None
        } else {
            Some(return_description_buff)
        };

        if arguments_descriptions.is_empty() && return_description.is_none() {
            return Ok(Function {
                description: s.to_string(),
                arguments_descriptions,
                return_description,
            });
        }

        Ok(Function {
            description: description_buff,
            arguments_descriptions,
            return_description,
        })
    }
}

/// Used to keep track of the different 
/// function comment parts while parsing it.
enum ParseStage {
    Description,
    Arguments,
    ReturnDescription,
}

/// Record or enum or object documentation.
#[derive(Debug, Clone)]
pub struct Structure {
    pub description: String,

    /// Methods documentation - empty for records and enums.
    pub methods: HashMap<String, Function>,
}

/// Impl documentation.
#[derive(Debug)]
struct Impl {
    methods: HashMap<String, Function>,
}

#[derive(Debug)]
pub struct Documentation {
    pub functions: HashMap<String, Function>,
    pub structures: HashMap<String, Structure>,
}

/// Extract doc comment from attributes.
///
/// Rust doc comments are silently converted (during parsing) to attributes of form:
/// #[doc = "documentation comment content"]
fn extract_doc_comment(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            attr.parse_meta().ok().and_then(|meta| {
                if let syn::Meta::NameValue(named_value) = meta {
                    let is_doc = named_value.path.is_ident("doc");
                    if is_doc {
                        match named_value.lit {
                            syn::Lit::Str(comment) => Some(comment.value().trim().to_string()),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
        .next()
}

/// Extract code documentation comments from Rust `lib.rs` file.
pub fn extract_documentation(path: &Utf8Path) -> Result<Documentation> {
    let input = read_to_string(path)?;
    let file = syn::parse_file(&input)?;

    let mut functions = HashMap::new();
    let mut structures = HashMap::new();
    let mut impls = HashMap::new();

    for item in file.items.into_iter() {
        match item {
            syn::Item::Enum(item) => {
                let name = item.ident.to_string();
                let description = extract_doc_comment(&item.attrs);
                if let Some(description) = description {
                    structures.insert(
                        name,
                        Structure {
                            description,
                            methods: HashMap::default(),
                        },
                    );
                }
            }
            syn::Item::Struct(item) => {
                let name = item.ident.to_string();
                let description = extract_doc_comment(&item.attrs);
                if let Some(description) = description {
                    structures.insert(
                        name,
                        Structure {
                            description,
                            methods: HashMap::default(),
                        },
                    );
                }
            }
            syn::Item::Impl(item) => {
                if item.trait_.is_none() {
                    if let syn::Type::Path(path) = *item.self_ty {
                        let name = path.path.segments[0].ident.to_string();

                        let methods = item
                            .items
                            .into_iter()
                            .filter_map(|item| {
                                if let syn::ImplItem::Method(method) = item {
                                    let name = method.sig.ident.to_string();
                                    extract_doc_comment(&method.attrs).map(|doc| (name, doc))
                                } else {
                                    None
                                }
                            })
                            .map(|(name, description)| {
                                (name, Function::from_str(&description).unwrap())
                            })
                            .collect();

                        impls.insert(name, Impl { methods });
                    }
                }
            }
            syn::Item::Fn(item) => {
                let name = item.sig.ident.to_string();
                let description = extract_doc_comment(&item.attrs);
                if let Some(description) = description {
                    functions.insert(
                        name,
                        Function {
                            description,
                            arguments_descriptions: HashMap::new(),
                            return_description: None,
                        },
                    );
                }
            }
            _ => (), // other item types are ignored,
        }
    }

    for (name, impl_) in impls {
        if let Some(structure) = structures.get_mut(&name) {
            structure.methods = impl_.methods;
        }
    }

    Ok(Documentation {
        functions,
        structures,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_doc_function_parses_a_md_description() {
        let description = indoc! {"
            This is the function description.
            Here is a second line.
            
            # Arguments
            
            - `argument1` - this is argument description 1.
            - `argument2` - this is argument description 2.
            
            # Returns
            
            This is return value description.
            Here is a second line.
        "};

        let result = Function::from_str(&description).unwrap();
        assert_eq!(expected_complete_doc_function(), result);
    }

    fn expected_complete_doc_function() -> Function {
        let mut expected_arg_descriptions = HashMap::new();
        expected_arg_descriptions.insert(
            "argument1".to_string(),
            "this is argument description 1.".to_string(),
        );
        expected_arg_descriptions.insert(
            "argument2".to_string(),
            "this is argument description 2.".to_string(),
        );
        Function {
            description: "This is the function description.\nHere is a second line.\n".to_string(),
            arguments_descriptions: expected_arg_descriptions,
            return_description: Some(
                "This is return value description.\nHere is a second line.\n".to_string(),
            ),
        }
    }

    #[test]
    fn test_doc_function_parses_a_no_md_description() {
        let description = indoc! {"
            This is the function description.

            Arguments

            argument1 - this is argument description 1.
            argument2 - this is argument description 2.

            Returns

            This is return value description.
        "};

        let result = Function::from_str(&description).unwrap();

        assert_eq!(
            Function {
                description: description.to_string(),
                arguments_descriptions: HashMap::new(),
                return_description: None
            },
            result
        );
    }
}
