/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_enum(input: general::Enum, context: &Context) -> Result<Enum> {
    let mut context = context.clone();
    context.update_from_enum(&input);

    let mut base_classes = vec![];
    let self_type = input.self_type.map_node(&context)?;
    let discr_type = input.discr_type.map_node(&context)?;

    let kotlin_kind = if matches!(input.shape, EnumShape::Error { flat: true }) {
        KotlinEnumKind::FlatError
    } else if self_type.is_used_as_error || !input.is_flat {
        if self_type.is_used_as_error {
            base_classes.push("Exception()".to_string());
        }
        if input.uniffi_trait_methods.ord_cmp.is_some() {
            base_classes.push(format!("Comparable<{}>", self_type.type_kt));
        }
        KotlinEnumKind::SealedClass
    } else {
        KotlinEnumKind::EnumClass {
            discr_type: input.discr_specified.then(|| discr_type.type_kt.clone()),
        }
    };

    let ffi_types = context.ffi_type_oracle.get_ffi_types(&self_type.ty)?;
    let ffi_fields: Vec<FfiField> = ffi_types
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, ty)| FfiField { index, ty })
        .collect();

    let variants = input
        .variants
        .into_iter()
        .map(|v| map_variant(v, &input.name, &ffi_fields, &context))
        .collect::<Result<Vec<_>>>()?;

    Ok(Enum {
        is_flat: input.is_flat,
        use_entries: context.config()?.use_enum_entries(),
        self_type,
        kotlin_kind,
        discr_type,
        discr_specified: input.discr_specified,
        variants,
        name: input.name,
        orig_name: input.orig_name,
        base_classes,
        uniffi_trait_methods: input.uniffi_trait_methods.map_node(&context)?,
        shape: input.shape,
        docstring: input.docstring,
        recursive: input.recursive,
        ffi_fields,
    })
}

pub fn map_variant(
    input: general::Variant,
    enum_name: &str,
    enum_ffi_fields: &[FfiField],
    context: &Context,
) -> Result<Variant> {
    let name_kt = variant_name_kt(&input, context)?;
    let mut field_alloc = VariantFieldAllocator::new(enum_name, enum_ffi_fields);
    let fields = map_fields(input.fields, &mut field_alloc, context)?;

    Ok(Variant {
        name_kt,
        name: input.name,
        orig_name: input.orig_name,
        discr: input.discr.map_node(context)?,
        fields_kind: input.fields_kind,
        fields,
        docstring: input.docstring,
        used_ffi_fields: field_alloc.used_ffi_fields,
    })
}

fn map_fields(
    input: Vec<general::Field>,
    field_alloc: &mut VariantFieldAllocator,
    context: &Context,
) -> Result<Vec<Field>> {
    let mut layout_builder = FfiBufferLayoutBuilder::new();
    layout_builder.extend(&Type::Int32, context)?; // discriminant
    input
        .into_iter()
        .enumerate()
        .map(|(index, input)| {
            let ty = input.ty.map_node(context)?;
            let ffi_fields = ty
                .ffi_types
                .iter()
                .map(|ffi_type| field_alloc.alloc(ffi_type))
                .collect::<Result<Vec<_>>>()?;
            let offset = layout_builder.extend(&ty.ty, context)?;

            Ok(Field {
                name: input.name,
                orig_name: input.orig_name,
                index,
                ty,
                default: input.default.map_node(context)?,
                docstring: input.docstring,
                ffi_fields,
                offset,
            })
        })
        .collect::<Result<Vec<_>>>()
}

/// Allocates FFI fields from the enum to the variant
struct VariantFieldAllocator<'a> {
    enum_name: &'a str,
    available_ffi_fields: Vec<FfiField>,
    used_ffi_fields: IndexSet<FfiField>,
}

impl<'a> VariantFieldAllocator<'a> {
    fn new(enum_name: &'a str, enum_ffi_fields: &[FfiField]) -> Self {
        // Skip the first enum FFI field, since that's the discriminant
        let available_ffi_fields = enum_ffi_fields.iter().skip(1).cloned().collect();
        let used_ffi_fields = IndexSet::from_iter([enum_ffi_fields[0]]);
        Self {
            enum_name,
            available_ffi_fields,
            used_ffi_fields,
        }
    }

    fn alloc(&mut self, ffi_type: &FfiType) -> Result<FfiField> {
        match self
            .available_ffi_fields
            .iter()
            .position(|f| f.ty == *ffi_type)
        {
            None => bail!(
                "UniFFI internal error: can't allocate FFI fields for {}",
                self.enum_name,
            ),
            Some(i) => {
                let ffi_field = self.available_ffi_fields.swap_remove(i);
                self.used_ffi_fields.insert(ffi_field);
                Ok(ffi_field)
            }
        }
    }
}

pub fn variant_name_kt(variant: &general::Variant, context: &Context) -> Result<String> {
    let en = context.current_enum()?;
    Ok(
        if !en.is_flat || matches!(en.shape, EnumShape::Error { flat: true }) {
            names::class_name_kt(&variant.name, en.self_type.is_used_as_error)
        } else {
            format!("`{}`", variant.name.to_shouty_snake_case())
        },
    )
}

impl Enum {
    pub fn name_kt(&self) -> String {
        names::class_name_kt(&self.name, self.self_type.is_used_as_error)
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.orig_name)
    }

    pub fn is_flat_error(&self) -> bool {
        matches!(self.shape, EnumShape::Error { flat: true })
    }
}

impl Variant {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.orig_name)
    }
}
