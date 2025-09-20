use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse::ParseStream, Data, DataStruct, DeriveInput, Field, LitStr, Token};

use crate::{
    default::{default_value_metadata_calls, DefaultValue},
    ffiops,
    util::{
        create_metadata_items, either_attribute_arg, extract_docstring, ident_to_string, kw,
        mod_path, try_metadata_value_from_usize, try_read_field, AttributeSliceExt,
        UniffiAttributeArgs,
    },
    DeriveOptions,
};

/// Handle #[uniffi(...)] attributes for records
#[derive(Clone, Default)]
pub struct RecordAttr {
    pub name: Option<String>,
}

impl UniffiAttributeArgs for RecordAttr {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            let _: kw::name = input.parse()?;
            let _: Token![=] = input.parse()?;
            let name = Some(input.parse::<LitStr>()?.value());
            Ok(Self { name })
        } else {
            Err(syn::Error::new(
                input.span(),
                format!("attribute `{input}` is not supported here."),
            ))
        }
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            name: either_attribute_arg(self.name, other.name)?,
        })
    }
}

/// Stores parsed data from the Derive Input for the struct.
struct RecordItem {
    ident: Ident,
    record: DataStruct,
    docstring: String,
    attr: RecordAttr,
}

impl RecordItem {
    fn new(input: DeriveInput) -> syn::Result<Self> {
        let record = match input.data {
            Data::Struct(s) => s,
            _ => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "This derive must only be used on structs",
                ));
            }
        };
        let attr = input.attrs.parse_uniffi_attr_args::<RecordAttr>()?;
        Ok(Self {
            ident: input.ident,
            record,
            docstring: extract_docstring(&input.attrs)?,
            attr,
        })
    }

    fn ident(&self) -> &Ident {
        &self.ident
    }

    fn foreign_name(&self) -> String {
        match &self.attr.name {
            Some(name) => name.clone(),
            None => ident_to_string(&self.ident),
        }
    }

    fn struct_(&self) -> &DataStruct {
        &self.record
    }

    fn docstring(&self) -> &str {
        self.docstring.as_str()
    }
}

pub fn expand_record(input: DeriveInput, options: DeriveOptions) -> syn::Result<TokenStream> {
    let record = RecordItem::new(input)?;
    let ffi_converter =
        record_ffi_converter_impl(&record, &options).unwrap_or_else(syn::Error::into_compile_error);
    let meta_static_var = options
        .generate_metadata
        .then(|| record_meta_static_var(&record).unwrap_or_else(syn::Error::into_compile_error));

    Ok(quote! {
        #ffi_converter
        #meta_static_var
    })
}

fn record_ffi_converter_impl(
    record: &RecordItem,
    options: &DeriveOptions,
) -> syn::Result<TokenStream> {
    let ident = record.ident();
    let impl_spec = options.ffi_impl_header("FfiConverter", ident);
    let derive_ffi_traits = options.derive_all_ffi_traits(ident);
    let name = &record.foreign_name();
    let mod_path = mod_path()?;
    let write_impl: TokenStream = record.struct_().fields.iter().map(write_field).collect();
    let try_read_fields: TokenStream = record.struct_().fields.iter().map(try_read_field).collect();

    Ok(quote! {
        #[automatically_derived]
        unsafe #impl_spec {
            ::uniffi::ffi_converter_rust_buffer_lift_and_lower!(crate::UniFfiTag);

            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                #write_impl
            }

            fn try_read(buf: &mut &[::std::primitive::u8]) -> ::uniffi::deps::anyhow::Result<Self> {
                ::std::result::Result::Ok(Self { #try_read_fields })
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_RECORD)
                .concat_str(#mod_path)
                .concat_str(#name);
        }

        #derive_ffi_traits
    })
}

fn write_field(f: &Field) -> TokenStream {
    let ident = &f.ident;
    let write = ffiops::write(&f.ty);
    quote! {
        #write(obj.#ident, buf);
    }
}

#[derive(Default)]
pub struct FieldAttributeArguments {
    pub(crate) default: Option<DefaultValue>,
}

impl UniffiAttributeArgs for FieldAttributeArguments {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: kw::default = input.parse()?;
        let lookahead = input.lookahead1();
        let default = if lookahead.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            input.parse()?
        } else {
            DefaultValue::Default
        };
        Ok(Self {
            default: Some(default),
        })
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            default: either_attribute_arg(self.default, other.default)?,
        })
    }
}

fn record_meta_static_var(record: &RecordItem) -> syn::Result<TokenStream> {
    let name = &record.foreign_name();
    let docstring = record.docstring();
    let module_path = mod_path()?;
    let fields_len = try_metadata_value_from_usize(
        record.struct_().fields.len(),
        "UniFFI limits structs to 256 fields",
    )?;

    let concat_fields: TokenStream = record
        .struct_()
        .fields
        .iter()
        .map(|f| {
            let attrs = f
                .attrs
                .parse_uniffi_attr_args::<FieldAttributeArguments>()?;

            let name = ident_to_string(f.ident.as_ref().unwrap());
            let docstring = extract_docstring(&f.attrs)?;
            let default = default_value_metadata_calls(&attrs.default)?;
            let type_id_meta = ffiops::type_id_meta(&f.ty);

            // Note: fields need to implement both `Lower` and `Lift` to be used in a record.  The
            // TYPE_ID_META should be the same for both traits.
            Ok(quote! {
                .concat_str(#name)
                .concat(#type_id_meta)
                #default
                .concat_long_str(#docstring)
            })
        })
        .collect::<syn::Result<_>>()?;

    Ok(create_metadata_items(
        "record",
        name,
        quote! {
            ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::RECORD)
                .concat_str(#module_path)
                .concat_str(#name)
                .concat_value(#fields_len)
                #concat_fields
                .concat_long_str(#docstring)
        },
        None,
    ))
}
