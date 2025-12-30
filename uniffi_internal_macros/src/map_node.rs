/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::collections::HashSet;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Member, Path,
    Result, Token, Type,
};

pub fn expand_derive(input: DeriveInput) -> Result<TokenStream> {
    match input.data {
        Data::Struct(st) => {
            let st = ParsedStruct::new(input.ident, input.attrs, st)?;
            let map_node_impls = st.render_map_prev_node_impls();
            let auto_map_impl = st.render_auto_map();
            Ok(quote! {
                #(
                    #map_node_impls
                )*

                #auto_map_impl
            })
        }
        Data::Enum(en) => {
            let en = ParsedEnum::new(input.ident, input.attrs, en)?;
            let map_node_impls = en.render_map_prev_node_impls();
            let auto_map_impl = en.render_auto_map();
            Ok(quote! {
                #(
                    #map_node_impls
                )*

                #auto_map_impl
            })
        }
        Data::Union(_) => panic!("#[derive(Node)] is not supported for unions"),
    }
}

struct ParsedStruct {
    ident: Ident,
    attrs: StructAttrs,
    fields: ParsedFields,
}

impl ParsedStruct {
    fn new(ident: Ident, attrs: Vec<Attribute>, st: DataStruct) -> Result<Self> {
        Ok(Self {
            attrs: StructAttrs::new(attrs)?,
            fields: ParsedFields::new(st.fields)?,
            ident,
        })
    }

    fn render_map_prev_node_impls(&self) -> Vec<TokenStream> {
        self.attrs
            .from
            .iter()
            .map(|input_node| {
                let output_node = &self.ident;

                if let Some(map_fn) = &self.attrs.map_fn {
                    return quote!{
                        #[automatically_derived]
                        impl ::uniffi_pipeline::MapNode<#output_node, Context> for #input_node {
                            fn map_node(self, context: &Context) -> ::uniffi_pipeline::Result<#output_node> {
                                #map_fn(self, context)
                            }
                        }
                    }
                }

                let maybe_clone_context = (!self.attrs.update_context.is_empty()).then(|| quote! {
                    let mut child_context: Context = context.clone();
                    let context = &mut child_context;
                });
                let update_context = &self.attrs.update_context;
                let output_members = self.fields.output_members();
                let exprs = std::iter::zip(self.fields.iter(), self.fields.input_members()).map(
                    |((attrs, _), member)| match &attrs.expr {
                        Some(expr) => quote! { #expr },
                        None => quote! { self.#member.map_node(context)?},
                    },
                );

                quote! {
                    #[automatically_derived]
                    impl ::uniffi_pipeline::MapNode<#output_node, Context> for #input_node {
                        fn map_node(self, context: &Context) -> ::uniffi_pipeline::Result<#output_node> {
                            #maybe_clone_context
                            #(
                                #update_context;
                            )*

                            Ok(#output_node {
                                #(
                                    #output_members: #exprs,
                                )*
                            })
                        }
                    }
                }
            })
            .collect()
    }

    /// Render `uniffi_auto_map_node`, which implements `MapNode::map_node` for any context type
    ///
    /// Why not just generate a blanket impl?  Because that adds too many impls, which makes the
    /// type hints worse.  For example, when you're mapping `Vec<PrevFoo>` -> `Vec<Foo>` it helps
    /// if the only `MapNode` impl for the current `Context` type maps to `Foo`.  Having a second
    /// impl that could map it to `PrevFoo` can result in type hints that are harder to follow.
    ///
    /// So instead, we always implement `render_auto_map`, but only use it as part of the `use_prev_node!` impl.
    fn render_auto_map(&self) -> TokenStream {
        let output_node = &self.ident;
        let output_members = self.fields.output_members();
        let unique_types = self.fields.unique_types();
        let generic_constraints = unique_types
            .iter()
            .map(|ty| quote! { #ty: uniffi_pipeline::MapNode<#ty, C> });

        quote! {
            #[automatically_derived]
            impl #output_node {
                pub fn uniffi_auto_map_node<C>(self, context: &C) -> ::uniffi_pipeline::Result<Self>
                    where #(#generic_constraints,)*
                {
                    Ok(Self {
                        #(
                            #output_members: self.#output_members.map_node(context)?,
                        )*
                    })
                }
            }
        }
    }
}

struct ParsedEnum {
    ident: Ident,
    attrs: EnumAttrs,
    variants: ParsedVariants,
}

impl ParsedEnum {
    fn new(ident: Ident, attrs: Vec<Attribute>, en: DataEnum) -> Result<Self> {
        Ok(Self {
            attrs: EnumAttrs::new(attrs)?,
            variants: ParsedVariants::new(en)?,
            ident,
        })
    }

    fn render_map_prev_node_impls(&self) -> Vec<TokenStream> {
        self.attrs
            .from
            .iter()
            .map(|input_node|{
                let output_node = &self.ident;

                if let Some(map_fn) = &self.attrs.map_fn {
                    return quote!{
                        #[automatically_derived]
                        impl ::uniffi_pipeline::MapNode<#output_node, Context> for #input_node {
                            fn map_node(self, context: &Context) -> ::uniffi_pipeline::Result<#output_node> {
                                #map_fn(self, context)
                            }
                        }
                    }
                }


                let maybe_clone_context = (!self.attrs.update_context.is_empty()).then(|| quote! {
                    let mut child_context = context.clone();
                    let context = &mut child_context;
                });
                let update_context = &self.attrs.update_context;

                let arms = self
                    .variants
                    .iter()
                    .filter(|(_, attrs, _)| !attrs.added)
                    .map(|(output_variant, attrs, fields)| {
                        let input_variant = attrs.from.as_ref().unwrap_or(output_variant);
                        let members = fields.output_members();
                        let pattern = fields.pattern();
                        let exprs =
                            std::iter::zip(fields.iter(), fields.var_names()).map(
                                |((attrs, _), var)| match &attrs.expr {
                                    Some(expr) => quote! { #expr },
                                    None => quote! { #var.map_node(context)?},
                                },
                            );

                        quote! {
                            #input_node::#input_variant #pattern => {
                                #output_node::#output_variant {
                                    #(
                                        #members: #exprs,
                                    )*
                                }
                            }
                        }
                    });

                quote! {
                    #[automatically_derived]
                    impl ::uniffi_pipeline::MapNode<#output_node, Context> for #input_node {
                        fn map_node(self, context: &Context) -> ::uniffi_pipeline::Result<#output_node> {
                            #maybe_clone_context
                            #(
                                #update_context;
                            )*
                            Ok(match self {
                                #(
                                    #arms
                                )*
                            })
                        }
                    }
                }
            })
            .collect()
    }

    fn render_auto_map(&self) -> TokenStream {
        let output_node = &self.ident;
        let unique_types = self.variants.unique_types();
        let generic_constraints = unique_types
            .iter()
            .map(|ty| quote! { #ty: uniffi_pipeline::MapNode<#ty, C> });

        let arms = self.variants.iter().map(|(variant, _, fields)| {
            let members = fields.output_members();
            let pattern = fields.pattern();
            let vars = fields.var_names();

            quote! {
                Self::#variant #pattern => {
                    Self::#variant {
                        #(
                            #members: #vars.map_node(context)?,
                        )*
                    }
                }
            }
        });

        quote! {
            #[automatically_derived]
            impl #output_node {
                pub fn uniffi_auto_map_node<C>(self, context: &C) -> ::uniffi_pipeline::Result<Self>
                    where #(#generic_constraints,)*
                {
                    Ok(match self {
                        #(
                            #arms
                        )*
                    })
                }
            }
        }
    }
}

#[derive(Clone, Default)]
struct StructAttrs {
    from: Vec<Type>,
    update_context: Vec<Expr>,
    map_fn: Option<Path>,
}

impl StructAttrs {
    fn new(attrs: Vec<Attribute>) -> Result<Self> {
        let mut parsed = StructAttrs::default();

        for attr in attrs {
            if attr.path().is_ident("map_node") {
                let type_attr: TypeAttr = attr.parse_args()?;
                match type_attr {
                    TypeAttr::From(ty) => parsed.from.push(ty),
                    TypeAttr::UpdateContext(expr) => parsed.update_context.push(expr),
                    TypeAttr::MapFn(p) => parsed.map_fn = Some(p),
                }
            }
        }
        Ok(parsed)
    }
}

#[derive(Clone, Default)]
struct EnumAttrs {
    from: Vec<Type>,
    update_context: Vec<Expr>,
    map_fn: Option<Path>,
}

impl EnumAttrs {
    fn new(attrs: Vec<Attribute>) -> Result<Self> {
        let mut parsed = EnumAttrs::default();
        for attr in attrs {
            if attr.path().is_ident("map_node") {
                let type_attr: TypeAttr = attr.parse_args()?;
                match type_attr {
                    TypeAttr::From(ty) => parsed.from.push(ty),
                    TypeAttr::UpdateContext(expr) => parsed.update_context.push(expr),
                    TypeAttr::MapFn(e) => parsed.map_fn = Some(e),
                }
            }
        }
        Ok(parsed)
    }
}

#[derive(Default, Clone)]
struct VariantAttrs {
    from: Option<Ident>,
    added: bool,
}

impl VariantAttrs {
    fn new(attrs: &[Attribute]) -> Result<Self> {
        let mut parsed = Self::default();
        for attr in attrs {
            if attr.path().is_ident("map_node") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("from") {
                        let content;
                        parenthesized!(content in meta.input);
                        parsed.from = Some(content.parse()?);
                        Ok(())
                    } else if meta.path.is_ident("added") {
                        parsed.added = true;
                        Ok(())
                    } else {
                        Err(meta.error("invalid node attr"))
                    }
                })?;
            }
        }
        Ok(parsed)
    }
}

#[derive(Default, Clone)]
struct FieldAttrs {
    from: Option<Ident>,
    expr: Option<Expr>,
}

impl FieldAttrs {
    fn new(attrs: &[Attribute]) -> Result<Self> {
        let mut parsed = Self::default();
        for attr in attrs {
            if attr.path().is_ident("map_node") {
                let field_attr: FieldAttr = attr.parse_args()?;
                match field_attr {
                    FieldAttr::From(i) => parsed.from = Some(i),
                    FieldAttr::Expr(e) => parsed.expr = Some(e),
                }
            }
        }
        Ok(parsed)
    }
}

enum TypeAttr {
    From(Type),
    UpdateContext(Expr),
    MapFn(Path),
}

impl Parse for TypeAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::from) {
            let _: kw::from = input.parse()?;
            let content;
            parenthesized!(content in input);
            Ok(Self::From(content.parse()?))
        } else if input.peek(kw::update_context) {
            let _: kw::update_context = input.parse()?;
            let content;
            parenthesized!(content in input);
            Ok(Self::UpdateContext(content.parse()?))
        } else {
            Ok(Self::MapFn(input.parse()?))
        }
    }
}

enum FieldAttr {
    From(Ident),
    Expr(Expr),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::from) {
            let _: kw::from = input.parse()?;
            let content;
            parenthesized!(content in input);
            Ok(Self::From(content.parse()?))
        } else {
            Ok(Self::Expr(input.parse()?))
        }
    }
}

enum ParsedFields {
    Named(Vec<(FieldAttrs, Field)>),
    Unnamed(Vec<(FieldAttrs, Field)>),
    Unit,
}

impl ParsedFields {
    fn new(f: Fields) -> Result<Self> {
        Ok(match f {
            Fields::Named(fields) => Self::Named(
                fields
                    .named
                    .into_iter()
                    .map(|f| Ok((FieldAttrs::new(&f.attrs)?, f)))
                    .collect::<Result<Vec<_>>>()?,
            ),
            Fields::Unnamed(fields) => Self::Unnamed(
                fields
                    .unnamed
                    .into_iter()
                    .map(|f| Ok((FieldAttrs::new(&f.attrs)?, f)))
                    .collect::<Result<Vec<_>>>()?,
            ),
            Fields::Unit => Self::Unit,
        })
    }

    fn iter(&self) -> impl Iterator<Item = &(FieldAttrs, Field)> {
        let (named_fields, unnamed_fields) = match self {
            Self::Named(fields) => (Some(fields), None),
            Self::Unnamed(fields) => (None, Some(fields)),
            Self::Unit => (None, None),
        };

        named_fields
            .into_iter()
            .flat_map(|f| f.iter())
            .chain(unnamed_fields.into_iter().flat_map(|f| f.iter()))
    }

    fn iter_fields(&self) -> impl Iterator<Item = &Field> {
        self.iter().map(|(_, f)| f)
    }

    fn types(&self) -> impl Iterator<Item = &Type> {
        self.iter().map(|(_, f)| &f.ty)
    }

    fn unique_types(&self) -> Vec<&Type> {
        let mut seen = HashSet::<&Type>::default();
        let mut types = vec![];
        for t in self.types() {
            if seen.insert(t) {
                types.push(t);
            }
        }
        types
    }

    fn input_members(&self) -> impl Iterator<Item = Member> + '_ {
        self.iter().enumerate().map(move |(i, (attrs, f))| {
            if self.is_named() {
                Member::Named(attrs.from.as_ref().or(f.ident.as_ref()).unwrap().clone())
            } else {
                Member::Unnamed(i.into())
            }
        })
    }

    fn output_members(&self) -> impl Iterator<Item = Member> + '_ {
        self.iter_fields().enumerate().map(move |(i, f)| {
            if self.is_named() {
                Member::Named(f.ident.clone().unwrap())
            } else {
                Member::Unnamed(i.into())
            }
        })
    }

    // Unique ident for each field
    fn var_names(&self) -> impl Iterator<Item = Ident> + use<'_> {
        let is_named = self.is_named();
        self.iter_fields().enumerate().map(move |(i, f)| {
            if is_named {
                f.ident.clone().unwrap()
            } else {
                format_ident!("var{i}")
            }
        })
    }

    // Pattern to match each field of an enum variant into `var_names`
    fn pattern(&self) -> TokenStream {
        let var_names = self.var_names();
        match self {
            Self::Named(_) => quote! { { #(#var_names),* } },
            Self::Unnamed(_) => quote! { ( #(#var_names),* ) },
            Self::Unit => quote! {},
        }
    }

    fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
    }
}

#[derive(Default)]
struct ParsedVariants {
    variants: Vec<(Ident, VariantAttrs, ParsedFields)>,
}

impl ParsedVariants {
    fn new(en: DataEnum) -> Result<Self> {
        let variants = en
            .variants
            .into_iter()
            .map(|v| {
                Ok((
                    v.ident,
                    VariantAttrs::new(&v.attrs)?,
                    ParsedFields::new(v.fields)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { variants })
    }

    fn iter(&self) -> impl Iterator<Item = &(Ident, VariantAttrs, ParsedFields)> {
        self.variants.iter()
    }

    fn unique_types(&self) -> Vec<&Type> {
        let mut seen = HashSet::<&Type>::default();
        let mut types = vec![];
        for (_, _, fields) in self.variants.iter() {
            for t in fields.types() {
                if seen.insert(t) {
                    types.push(t);
                }
            }
        }
        types
    }
}

impl ToTokens for ParsedFields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = self.iter().map(|(_, field)| field);
        match self {
            Self::Named(_) => tokens.extend(quote! { { #(#fields,)* } }),
            Self::Unnamed(_) => tokens.extend(quote! { ( #(#fields,)* ) }),
            Self::Unit => (),
        }
    }
}

impl ToTokens for ParsedVariants {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for (ident, _, fields) in self.iter() {
            ident.to_tokens(tokens);
            fields.to_tokens(tokens);
            tokens.extend(quote! { , });
        }
    }
}

pub fn expand_use_prev_node(input: PrevNodeInput) -> Result<TokenStream> {
    let PrevNodeInput { node_path, map_fn } = input;
    let ident = match node_path.segments.last() {
        Some(i) => i,
        None => return Err(syn::Error::new_spanned(node_path, "Invalid input")),
    };
    let expr = match map_fn {
        None => quote! { Self::uniffi_auto_map_node::<Context>(self, context) },
        Some(path) => quote! { #path(self, context) },
    };

    Ok(quote! {
        pub use #node_path;

        #[automatically_derived]
        impl ::uniffi_pipeline::MapNode<#ident, Context> for #ident {
            fn map_node(self, context: &Context) -> ::uniffi_pipeline::Result<#ident> {
                #expr
            }
        }
    })
}

pub struct PrevNodeInput {
    node_path: Path,
    map_fn: Option<Path>,
}

impl Parse for PrevNodeInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let node_path = input.parse()?;
        let mut map_fn = None;
        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            map_fn = Some(input.parse()?);
        }
        Ok(Self { node_path, map_fn })
    }
}

mod kw {
    syn::custom_keyword!(from);
    syn::custom_keyword!(update_context);
}
