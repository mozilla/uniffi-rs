/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{alloc::Layout, cmp::max};

use super::*;

/// Used to build FFI buffer layouts.
///
/// This is a wrapper around `std::alloc::Layout`.
/// See the "FFI buffers" section of `DESIGN.md` for details.
pub struct FfiBufferLayoutBuilder {
    current_layout: Layout,
}

impl FfiBufferLayoutBuilder {
    /// Get a new layout to use for calculating offsets
    pub fn new() -> Self {
        Self {
            current_layout: Layout::from_size_align(0, 1).unwrap(),
        }
    }

    pub fn size(&self) -> usize {
        self.current_layout.size()
    }

    /// Extend this layout with the layout for `ty`.
    ///
    /// This adds any padding needed to align the newly added layout.
    ///
    /// Returns the offset of the layout for `ty.
    pub fn extend(&mut self, ty: &Type, context: &Context) -> Result<usize> {
        self.extend_with_layout(context.layout_oracle.get(ty)?)
    }

    fn extend_with_layout(&mut self, layout: Layout) -> Result<usize> {
        let (new_layout, offset) = self.current_layout.extend(layout)?;
        self.current_layout = new_layout;
        Ok(offset)
    }

    pub fn pad_to_align(&mut self) {
        self.current_layout = self.current_layout.pad_to_align()
    }
}

/// Knows the layouts for all types in the interface
#[derive(Clone, Default)]
pub struct FfiBufferLayoutOracle {
    // Map user types to their layouts
    layout_map: HashMap<Type, Layout>,
}

impl FfiBufferLayoutOracle {
    // Get the layout for a type
    pub fn get(&self, ty: &Type) -> Result<Layout> {
        Ok(match ty {
            Type::UInt8 | Type::Int8 | Type::Boolean => Layout::from_size_align(1, 1)?,
            Type::UInt16 | Type::Int16 => Layout::from_size_align(2, 2)?,
            Type::UInt32 | Type::Int32 | Type::Float32 => Layout::from_size_align(4, 4)?,
            Type::UInt64 | Type::Int64 | Type::Float64 => Layout::from_size_align(8, 8)?,
            // One 8-byte handle
            Type::Interface { .. } => Layout::from_size_align(8, 8)?,
            // (data, length, capacity) fields
            Type::String => Layout::from_size_align(24, 8)?,
            // (data, size) fields
            Type::Sequence { .. } | Type::Map { .. } | Type::Set { .. } => {
                Layout::from_size_align(16, 8)?
            }
            // single data field
            Type::Box { .. } => Layout::from_size_align(8, 8)?,
            // Other types require a lookup from the layout map
            _ => match self.layout_map.get(ty) {
                Some(layout) => *layout,
                None => bail!("No FFI buffer Layout for {ty:?}"),
            },
        })
    }

    pub fn add_type_definitions(
        &mut self,
        sorted_type_definitions: &[general::TypeDefinition],
    ) -> Result<()> {
        for type_def in sorted_type_definitions {
            if self.layout_map.contains_key(type_def.self_type()) {
                continue;
            }

            match type_def {
                general::TypeDefinition::Record(rec) => self.add_record(rec)?,
                general::TypeDefinition::Enum(en) => self.add_enum(en)?,
                general::TypeDefinition::Optional(opt) => self.add_optional(opt)?,
                _ => (),
            }
        }
        Ok(())
    }

    fn layout_for_types<'a>(&self, types: impl IntoIterator<Item = &'a Type>) -> Result<Layout> {
        let mut layout = FfiBufferLayoutBuilder::new();
        for ty in types {
            layout.extend_with_layout(self.get(ty)?)?;
        }
        Ok(layout.current_layout)
    }

    fn add_record(&mut self, rec: &general::Record) -> Result<()> {
        let layout = self.layout_for_types(rec.fields.iter().map(|f| &f.ty.ty))?;
        self.layout_map.insert(rec.self_type.ty.clone(), layout);
        Ok(())
    }

    fn add_enum(&mut self, en: &general::Enum) -> Result<()> {
        let mut size = 0;
        let mut align = 1;

        for v in en.variants.iter() {
            let v_layout = self.layout_for_types(
                std::iter::once(&Type::Int32).chain(v.fields.iter().map(|f| &f.ty.ty)),
            )?;
            size = max(size, v_layout.size());
            align = max(align, v_layout.align());
        }

        self.layout_map.insert(
            en.self_type.ty.clone(),
            Layout::from_size_align(size, align)?,
        );
        Ok(())
    }

    fn add_optional(&mut self, opt: &general::OptionalType) -> Result<()> {
        let layout = self.layout_for_types([&Type::Boolean, &opt.inner.ty])?;
        self.layout_map.insert(opt.self_type.ty.clone(), layout);
        Ok(())
    }
}
