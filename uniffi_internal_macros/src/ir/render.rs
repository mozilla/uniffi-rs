/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Render AST nodes

use super::ast::*;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

impl Node {
    pub fn render_visit_children(&self) -> TokenStream {
        let body = self.render_visit_children_body();
        quote! {
            fn visit_children(&self, visitor: &mut dyn FnMut(&str, &dyn ::uniffi_pipeline::Node) -> ::uniffi_pipeline::Result<()>) -> ::uniffi_pipeline::Result<()>
            {
                use ::uniffi_pipeline::*;
                #body
            }
        }
    }

    pub fn render_visit_children_mut(&self) -> TokenStream {
        let body = self.render_visit_children_body();
        quote! {
            fn visit_children_mut(&mut self, visitor: &mut dyn FnMut(&str, &mut dyn ::uniffi_pipeline::Node) -> ::uniffi_pipeline::Result<()>) -> ::uniffi_pipeline::Result<()>
            {
                use ::uniffi_pipeline::*;
                #body
            }
        }
    }

    fn render_visit_children_body(&self) -> TokenStream {
        self.deconstruct_expr(|kind, fields| {
            let var_names = fields.var_names();
            let names = fields.iter().map(|f| kind.field_name(f));
            quote! {
                #(
                    visitor(#names, #var_names)?;
                )*
                Ok(())
            }
        })
    }

    pub fn render_take_into_value(&self) -> TokenStream {
        let type_name = self.ident.to_string();
        let body = self.deconstruct_expr(|kind, fields| {
            let field_names = fields.iter().enumerate().map(|(idx, f)| match &f.ident {
                Some(name) => name.to_string(),
                None => idx.to_string(),
            });
            let vars = fields.iter().map(|f| &f.var_name);
            match kind {
                NodeKind::Struct { .. } => {
                    quote! {
                        Value::Struct {
                            type_name: #type_name,
                            fields: ::std::collections::HashMap::from([
                                #(
                                    (#field_names, #vars.take_into_value()),
                                )*
                            ]),
                        }
                    }
                }
                NodeKind::Variant { variant_name, .. } => {
                    let variant_name = variant_name.to_string();
                    quote! {
                        Value::Variant {
                            type_name: #type_name,
                            variant_name: #variant_name,
                            fields: ::std::collections::HashMap::from([
                                #(
                                    (#field_names, #vars.take_into_value()),
                                )*
                            ]),
                        }
                    }
                }
            }
        });

        quote! {
            fn take_into_value(&mut self) -> ::uniffi_pipeline::Value {
                use ::uniffi_pipeline::*;
                #body
            }
        }
    }

    pub fn render_default(&self) -> TokenStream {
        let type_name = &self.ident;
        let body = match &self.def {
            NodeDef::Struct(st) => st.fields.construct_default(&NodeKind::Struct { type_name }),
            NodeDef::Enum(variants) => match variants.first() {
                Some(v) => {
                    let variant_name = &v.ident;
                    v.fields.construct_default(&NodeKind::Variant {
                        type_name,
                        variant_name,
                    })
                }
                None => quote! {
                    panic!("{} has no variants", #type_name)
                },
            },
        };

        quote! {
            fn default() -> Self {
                use ::uniffi_pipeline::*;
                #body
            }
        }
    }

    pub fn render_try_from_value(&self) -> TokenStream {
        if let NodeDef::Struct(Struct {
            wraps_field: Some(wraps_field),
            fields,
        }) = &self.def
        {
            return self.render_try_from_value_with_wraps_field(wraps_field, fields);
        }

        let type_name = &self.ident;
        let type_name_str = self.prev_node_ident().to_string();
        let cases = match &self.def {
            NodeDef::Struct(st) => {
                let construct = st
                    .fields
                    .try_construct_from_hash_map(&NodeKind::Struct { type_name });
                quote! {
                    Value::Struct { type_name, mut fields } if type_name == #type_name_str => {
                        #construct
                    }
                }
            }
            NodeDef::Enum(variants) => {
                let cases = variants.iter().map(|v| {
                    let variant_name = &v.ident;
                    let variant_name_str = v.prev_node_ident().to_string();
                    let construct = v.fields.try_construct_from_hash_map(&NodeKind::Variant {
                        type_name,
                        variant_name,
                    });
                    quote! {
                        Value::Variant { type_name, variant_name, mut fields }
                            if type_name == #type_name_str && variant_name == #variant_name_str => {
                            #construct
                        }
                    }
                });
                quote! { #(#cases,)* }
            }
        };

        quote! {
            fn try_from_value(value: ::uniffi_pipeline::Value) -> ::uniffi_pipeline::Result<Self, ::uniffi_pipeline::FromValueError> {
                use ::uniffi_pipeline::*;
                match value {
                    #cases
                    v => Err(FromValueError::new(
                        format!("expected {}, actual {v:?}", #type_name_str),
                    )),
                }
            }
        }
    }

    pub fn render_try_from_value_with_wraps_field(
        &self,
        wraps_field: &Field,
        fields: &Fields,
    ) -> TokenStream {
        let other_fields = fields
            .iter()
            .filter(|f| f.var_name != wraps_field.var_name)
            .map(|f| &f.var_name);
        let wraps_field_name = &wraps_field.var_name;

        quote! {
            fn try_from_value(value: ::uniffi_pipeline::Value) -> ::uniffi_pipeline::Result<Self, ::uniffi_pipeline::FromValueError> {
                use ::uniffi_pipeline::*;
                Ok(Self {
                    #wraps_field_name: value.try_into_node()?,
                    #(
                        #other_fields: ::std::default::Default::default(),
                    )*
                })
            }
        }
    }

    /// Renders an expression based on deconstructing the node into it's fields
    ///
    /// The main point of this is that it works the same for both structs and enums.
    /// Structs will be deconstructed into their fields, while enums will be used in a match
    /// statement. Either way the provided closure generates an expression based on the
    /// struct/variant fields.  The closure inputs:
    ///
    ///   * A path that represents the type name for structs or the type+variant name for enums
    ///   * `Fields` for the struct/variant
    ///
    /// Use [Field::var_name] to get a variable in the current scope for the field.
    pub fn deconstruct_expr(
        &self,
        mut expr_generator: impl FnMut(&NodeKind<'_>, &Fields) -> TokenStream,
    ) -> TokenStream {
        let type_name = &self.ident;
        match &self.def {
            NodeDef::Struct(st) => {
                let pattern = st.fields.pattern();
                let kind = NodeKind::Struct {
                    type_name: &self.ident,
                };
                let expr = expr_generator(&kind, &st.fields);
                quote! {
                    let #type_name #pattern = self;
                    #expr
                }
            }
            NodeDef::Enum(variants) => {
                let cases = variants.iter().map(|v| {
                    let pattern = v.fields.pattern();
                    let kind = NodeKind::Variant {
                        type_name: &self.ident,
                        variant_name: &v.ident,
                    };
                    let expr = expr_generator(&kind, &v.fields);
                    quote! {
                        #kind #pattern => {
                            #expr
                        }
                    }
                });

                quote! {
                    match self {
                        #(#cases)*
                    }
                }
            }
        }
    }

    pub fn split_generics_for_node_impl(&self) -> (TokenStream, TokenStream, TokenStream) {
        let generics = self.generics.clone();
        let mut predicates = match generics.where_clause.clone() {
            Some(w) => w
                .predicates
                .into_iter()
                .map(|p| p.to_token_stream())
                .collect(),
            None => vec![],
        };
        predicates.extend(generics.type_params().map(|ty| {
            let ident = &ty.ident;
            quote! {
                #ident: Node,
            }
        }));
        let where_clause = if predicates.is_empty() {
            TokenStream::default()
        } else {
            quote! { where #(#predicates),* }
        };

        let (impl_generics, ty_generics, _) = generics.split_for_impl();
        (
            impl_generics.to_token_stream(),
            ty_generics.to_token_stream(),
            where_clause,
        )
    }
}

impl ToTokens for NodeKind<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Struct { type_name } => type_name.to_tokens(tokens),
            Self::Variant {
                type_name,
                variant_name,
            } => tokens.extend(quote! { #type_name::#variant_name }),
        }
    }
}

impl Fields {
    /// Construct a node using a HashMap
    ///
    /// This is used by the `try_from_value` function
    fn try_construct_from_hash_map(&self, kind: &NodeKind<'_>) -> TokenStream {
        let vars = self.iter().map(|f| &f.var_name);
        match self {
            Self::Named(fields) => {
                let keys = fields
                    .iter()
                    .map(|f| f.prev_node_ident().unwrap().to_string())
                    .collect::<Vec<_>>();
                let field_names = fields.iter().map(|f| kind.field_name(f));

                quote! {
                    Ok(#kind {
                        #(
                            #vars: match fields.remove(#keys) {
                                Some(v) => v.try_into_node().map_err(|e| e.add_field_to_path(#field_names))?,
                                None => ::std::default::Default::default(),
                            },
                        )*
                    })
                }
            }
            Self::Unnamed(fields) => {
                let keys = (0..fields.len()).map(|i| i.to_string());
                let field_names = fields.iter().map(|f| kind.field_name(f));
                quote! {
                    Ok(#kind (
                        #(
                            match fields.remove(#keys) {
                                Some(v) => v.try_into_node().map_err(|e| e.add_field_to_path(#field_names))?,
                                None => ::std::default::Default::default(),
                            },
                        )*
                    ))
                }
            }
            Self::Unit => quote! { Ok(#kind) },
        }
    }

    fn construct_default(&self, kind: &NodeKind<'_>) -> TokenStream {
        match self {
            Self::Named(fields) => {
                let fields = fields.iter().map(|f| {
                    let name = &f.var_name;
                    quote! { #name: ::std::default::Default::default() }
                });
                quote! {
                    #kind { #(#fields),* }
                }
            }
            Self::Unnamed(fields) => {
                let fields =
                    (0..fields.len()).map(|_| quote! { ::std::default::Default::default() });
                quote! {
                    #kind (#(#fields),*)
                }
            }
            Self::Unit => quote! { #kind },
        }
    }

    /// Generate a pattern to destructure a variant/struct
    ///
    /// This will not include the struct/variant idents, only the pattern for the fields
    pub fn pattern(&self) -> TokenStream {
        let var_names = self.iter().map(|f| &f.var_name);
        match self {
            Self::Unit => quote! {},
            Self::Named(_) => quote! { { #(#var_names,)* ..  } },
            Self::Unnamed(_) => quote! { ( #(#var_names),* ) },
        }
    }
}
