/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Render AST nodes

use super::{ast::*, ir_mod};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{ExprStruct, Ident, Member, Path};

impl DefineIrPassInput {
    pub fn render(mut self) -> syn::Result<TokenStream> {
        let ir_mod = ir_mod();
        let mut output = TokenStream::default();
        for (ident, to_node) in self.to_items.nodes {
            match self.from_items.nodes.shift_remove(&ident) {
                None => {
                    // Node added in the `to` IR, just copy it's definition to the output
                    output.extend(to_node.render(Ir::Pass));
                    output.extend(to_node.impl_from_node_for_output_type(&self.to));
                    output.extend(to_node.construct_macros());
                }
                Some(from_node) => {
                    // Node in both `from` and `to` IR.

                    // If we can, merge the two nodes together.  Otherwise, use the `to` node and
                    // require the user to implement `FromNode` manually so things work.
                    let node = to_node
                        .try_merge(from_node)
                        .unwrap_or_else(|to_node| to_node);
                    output.extend(node.render(Ir::Pass));
                    output.extend(node.impl_from_node_for_pass_type(&self.from));
                    output.extend(node.impl_from_node_for_output_type(&self.to));
                    output.extend(node.construct_macros());
                }
            }
        }
        // Anything left in `self.from_items` is a type in the `from` IR, but not the `to` IR.
        for from_node in self.from_items.nodes.into_values() {
            output.extend(from_node.render(Ir::Pass));
            output.extend(from_node.impl_from_node_for_pass_type(&self.from));
        }

        // Render the `to` impl block only.  The `from` one is likely to have type errors as the
        // types are transformed into the new IR.
        let impls = &self.to_items.impls;

        Ok(quote! {
            use ::indexmap::IndexMap;
            use #ir_mod::{bail, Node, FromNode, IntoNode};

            #output
            #(#impls)*
        })
    }
}

impl ConstructNodeInput {
    pub fn render(&self) -> TokenStream {
        let ExprStruct {
            attrs,
            path,
            fields,
            rest,
            ..
        } = &self.expr;
        if rest.is_some() {
            self.expr.to_token_stream()
        } else {
            let trailing_comma = (!fields.empty_or_trailing()).then(|| quote! { , });
            let missing_fields = self.fields.iter().filter(|ident| {
                !fields
                    .iter()
                    .any(|f| matches!(&f.member,  Member::Named(i) if i == *ident))
            });
            quote! {
                #(#attrs)*
                #path
                {
                    #fields
                    #trailing_comma
                    #(#missing_fields: Node::empty()),*
                }
            }
        }
    }
}

impl Node {
    pub fn render(&self, ir: Ir) -> TokenStream {
        let Self {
            ident, def, vis, ..
        } = self;
        let attrs = self.attrs.render(ir);

        match def {
            NodeDef::Struct(fields) => {
                let trailing_semi = fields.not_named().then_some(quote! { ; }).into_iter();
                let fields = fields.render(ir);
                quote! {
                    #attrs
                    #vis struct #ident #fields #(#trailing_semi)*
                }
            }
            NodeDef::Enum(variants) => {
                let variants = variants.variants.values().map(|v| v.render(ir));
                quote! {
                    #attrs
                    #vis enum #ident {
                        #( #variants,)*
                    }
                }
            }
        }
    }

    fn impl_from_node_for_pass_type(&self, input_module: &Path) -> TokenStream {
        let ir_mod = ir_mod();
        let type_name = &self.ident;
        let body = match &self.def {
            NodeDef::Struct(fields) => {
                let pattern = fields.pattern_from_ir();
                let construct = fields.construct_from_ir();
                quote! {
                    let #input_module::#type_name #pattern = value;
                    Ok(#type_name #construct)
                }
            }
            NodeDef::Enum(variants) => {
                let cases = variants.variants
                    .iter()
                    .filter(|(_, v)| v.in_from_ir())
                    .map(|(variant, v)| {
                        let pattern = v.fields.pattern_from_ir();
                        let construct = v.fields.construct_from_ir();
                        quote! {
                            #input_module::#type_name::#variant #pattern => Ok(#type_name::#variant #construct),
                        }
                    });

                quote! {
                    match value {
                        #(#cases)*
                    }
                }
            }
        };

        quote! {
            #[automatically_derived]
            impl #ir_mod::FromNode<#input_module::#type_name> for #type_name {
                fn from_node(value: #input_module::#type_name) -> #ir_mod::Result<#type_name> {
                    #body
                }
            }
        }
    }

    fn impl_from_node_for_output_type(&self, output_module: &Path) -> TokenStream {
        let ir_mod = ir_mod();
        let type_name = &self.ident;
        let body = match &self.def {
            NodeDef::Struct(fields) => {
                let pattern = fields.pattern();
                let construct = fields.construct_to_ir();
                quote! {
                    let #type_name #pattern = value;
                    Ok(#output_module::#type_name #construct)
                }
            }
            NodeDef::Enum(variants) => {
                let cases = variants.variants
                    .iter()
                    .filter(|(_, v)| v.in_to_ir())
                    .map(|(ident, v)| {
                        let pattern = v.fields.pattern();
                        let construct = v.fields.construct_to_ir();
                        quote! {
                            #type_name::#ident #pattern => Ok(#output_module::#type_name::#ident #construct),
                        }
                    });

                quote! {
                    match value {
                        #(#cases)*
                        value => bail!("Removed variant at end of pass: {value:?}"),
                    }
                }
            }
        };

        quote! {
            #[automatically_derived]
            impl #ir_mod::FromNode<#type_name> for #output_module::#type_name {
                fn from_node(value: #type_name) -> #ir_mod::Result<#output_module::#type_name> {
                    #body
                }
            }
        }
    }

    fn construct_macros(&self) -> TokenStream {
        let type_name = &self.ident;
        match &self.def {
            NodeDef::Struct(fields) => {
                let maybe_macro = fields.construct_macro(type_name, None);
                quote! { #maybe_macro }
            }
            NodeDef::Enum(e) => {
                let macros = e.variants.iter().filter_map(|(variant_name, v)| {
                    v.fields.construct_macro(type_name, Some(variant_name))
                });
                quote! { #(#macros)* }
            }
        }
    }
}

impl Attributes {
    pub fn render(&self, ir: Ir) -> TokenStream {
        let other = self.other.iter();
        let from_uniffi_meta = (ir == Ir::NonPass)
            .then(|| {
                self.from_uniffi_meta
                    .as_ref()
                    .map(|from_uniffi_meta| quote! { #[from_uniffi_meta(#from_uniffi_meta)] })
            })
            .flatten();
        let pass_only = (ir == Ir::PassInput && self.pass_only).then(|| quote! { #[pass_only] });

        quote! {
            #(#other)*
            #from_uniffi_meta
            #pass_only
        }
    }
}

impl Variant {
    pub fn render(&self, ir: Ir) -> TokenStream {
        let Self { vis, ident, .. } = self;
        let attrs = self.attrs.render(ir);
        let fields = self.fields.render(ir);
        let discriminant = self.discriminant.as_ref().map(|lit| quote! { = #lit });
        quote! { #attrs #vis #ident #fields #discriminant }
    }
}

impl Fields {
    pub fn render(&self, ir: Ir) -> TokenStream {
        match self {
            Self::Unit => TokenStream::default(),
            Self::Tuple(tuple_fields) => {
                let fields = tuple_fields
                    .iter()
                    .map(|TupleField { attrs, vis, ty, .. }| {
                        let attrs = attrs.render(ir);
                        quote! { #attrs #vis #ty }
                    });
                quote! { (#(#fields),*) }
            }
            Self::Named(fields) => {
                let fields = fields
                    .values()
                    .filter(|f| !(f.attrs.pass_only && ir == Ir::NonPass))
                    .map(|f| f.render(ir));
                quote! { { #(#fields),* } }
            }
        }
    }

    /// Generate a pattern to destructure a variant/struct
    ///
    /// This will not include the struct/variant idents, only the pattern for the fields
    pub fn pattern(&self) -> TokenStream {
        match self {
            Self::Unit => quote! {},
            Self::Named(fields) => {
                let source_fields = fields.keys();
                quote! { { #(#source_fields,)* ..  } }
            }
            Self::Tuple(fields) => {
                let var_names = fields.iter().map(|f| &f.var_name);
                quote! { ( #(#var_names),* ) }
            }
        }
    }

    /// Generate a pattern to destructure a variant/struct, from the previous type
    pub fn pattern_from_ir(&self) -> TokenStream {
        match self {
            Self::Unit => quote! {},
            Self::Named(fields) => {
                let source_fields = fields.values().filter(|f| f.in_from_ir()).map(|f| &f.ident);
                quote! { { #(#source_fields,)* ..  } }
            }
            Self::Tuple(fields) => {
                let var_names = fields.iter().map(|f| &f.var_name);
                quote! { ( #(#var_names),* ) }
            }
        }
    }

    /// Like `Fields::prev_type_pattern`, but this takes into account the `from_uniffi_meta` attribute
    pub fn uniffi_meta_type_pattern(&self) -> TokenStream {
        match self {
            Fields::Unit => quote! {},
            Fields::Named(fields) => {
                let source_fields = fields
                    .iter()
                    .map(|(ident, f)| f.attrs.from_uniffi_meta.as_ref().unwrap_or(ident));
                quote! { { #(#source_fields,)* ..  } }
            }
            Fields::Tuple(fields) => {
                let var_names = fields.iter().map(|f| &f.var_name);
                quote! { ( #(#var_names),* ) }
            }
        }
    }

    /// Generate a pattern to construct a variant/struct with these fields
    ///
    /// This will not include the struct/variant idents, only the pattern for the fields
    ///
    /// Inputs a closure that generates expressions for named fields.  For tuple fields, we always
    /// just call `into_node()`
    pub fn construct(
        &self,
        named_field_expr: impl Fn(&Field) -> Option<TokenStream>,
        tuple_field_expr: impl Fn(&TupleField) -> Option<TokenStream>,
    ) -> TokenStream {
        match self {
            Self::Unit => quote! {},
            Self::Named(fields) => {
                let field_exprs = fields.values().filter_map(|f| {
                    named_field_expr(f).map(|expr| {
                        let ident = &f.ident;
                        Some(quote! { #ident: #expr })
                    })
                });
                quote! { { #(#field_exprs,)* } }
            }
            Self::Tuple(fields) => {
                let exprs = fields.iter().filter_map(tuple_field_expr);
                quote! { ( #(#exprs,)* ) }
            }
        }
    }

    pub fn construct_from_ir(&self) -> TokenStream {
        self.construct(
            |field| {
                let ident = &field.ident;
                if field.in_from_ir() {
                    Some(quote! { #ident.into_node()? })
                } else {
                    Some(quote! { Node::empty() })
                }
            },
            |f| {
                let var = &f.var_name;
                Some(quote! { #var.into_node()? })
            },
        )
    }

    pub fn construct_from_uniffi_meta(&self) -> TokenStream {
        self.construct(
            |field| {
                let ident = field
                    .attrs
                    .from_uniffi_meta
                    .as_ref()
                    .unwrap_or(&field.ident);
                Some(quote! { #ident.into_node()? })
            },
            |f| {
                let var = &f.var_name;
                Some(quote! { #var.into_node()? })
            },
        )
    }

    pub fn construct_to_ir(&self) -> TokenStream {
        self.construct(
            |field| {
                let ident = &field.ident;
                if field.in_to_ir() {
                    Some(quote! { #ident.into_node()? })
                } else {
                    None
                }
            },
            |f| {
                let var = &f.var_name;
                Some(quote! { #var.into_node()? })
            },
        )
    }

    pub fn construct_empty(&self) -> TokenStream {
        self.construct(
            |_| Some(quote! { Node::empty() }),
            |_| Some(quote! { Node::empty() }),
        )
    }

    fn construct_macro(
        &self,
        type_name: &Ident,
        variant_name: Option<&Ident>,
    ) -> Option<TokenStream> {
        let name = match variant_name {
            None => format_ident!("{type_name}"),
            Some(variant_name) => format_ident!("{type_name}_{variant_name}"),
        };
        let node_path = match variant_name {
            None => quote! { #type_name },
            Some(variant_name) => quote! { #type_name::#variant_name },
        };
        match self {
            Self::Named(fields) => {
                let fields = fields.keys();
                Some(quote! {
                    #[allow(non_snake_case, unused)]
                    macro_rules! #name {
                        ($($tt:tt)*) => {
                            uniffi_internal_macros::construct_node!(
                                [#(#fields),*]
                                #node_path {
                                    $($tt)*
                                }
                            )
                        }
                    }
                })
            }
            _ => None,
        }
    }
}

impl Field {
    pub fn render(&self, ir: Ir) -> TokenStream {
        let Self {
            attrs,
            vis,
            ident,
            ty,
            ..
        } = self;
        let attrs = attrs.render(ir);
        quote! { #attrs #vis #ident: #ty }
    }
}

/// Which IR are we rendering?
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ir {
    // From/To IRs
    NonPass,
    // Pass IRs
    Pass,
    // Tokens for the `ir_pass!` input
    PassInput,
}
