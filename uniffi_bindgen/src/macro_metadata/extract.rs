/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{anyhow, bail, Context};
use camino::Utf8Path;
use fs_err as fs;
use goblin::{
    archive::Archive,
    elf::Elf,
    mach::{segment::Section, symbols, Mach, MachO, SingleArch},
    pe::{Coff, PE},
    Object,
};
use std::collections::HashSet;
use uniffi_meta::Metadata;
use walrus::{ir::Value, ConstExpr, DataKind, ExportItem, GlobalId, GlobalKind, Module};

/// Extract metadata written by the `uniffi::export` macro from a library file
///
/// In addition to generating the scaffolding, that macro and also encodes the
/// `uniffi_meta::Metadata` for the components which can be used to generate the bindings side of
/// the interface.
pub fn extract_from_library(path: &Utf8Path) -> anyhow::Result<Vec<Metadata>> {
    extract_from_bytes(&fs::read(path)?)
}

fn extract_from_bytes(file_data: &[u8]) -> anyhow::Result<Vec<Metadata>> {
    match Object::parse(file_data)? {
        Object::Elf(elf) => extract_from_elf(elf, file_data),
        Object::PE(pe) => extract_from_pe(pe, file_data),
        Object::Mach(mach) => extract_from_mach(mach, file_data),
        Object::Archive(archive) => extract_from_archive(archive, file_data),
        Object::COFF(coff) => extract_from_coff(coff, file_data),
        _ => match Module::from_buffer(file_data) {
            Ok(module) => extract_from_wasm(&module),
            Err(_) => bail!("Unknown library format"),
        },
    }
}

pub fn extract_from_elf(elf: Elf<'_>, file_data: &[u8]) -> anyhow::Result<Vec<Metadata>> {
    let mut extracted = ExtractedItems::new();

    // Some ELF files have a SHT_SYMTAB_SHNDX section that we use below.  If present, find the
    // offset for that section.
    let symtab_shndx_section_offset = elf
        .section_headers
        .iter()
        .find(|sh| sh.sh_type == goblin::elf::section_header::SHT_SYMTAB_SHNDX)
        .map(|sh| sh.sh_offset as usize);

    for (i, sym) in elf.syms.iter().enumerate() {
        let name = elf
            .strtab
            .get_at(sym.st_name)
            .context("Error getting symbol name")?;
        if !is_metadata_symbol(name) {
            continue;
        }

        let header_index = match sym.st_shndx as u32 {
            goblin::elf::section_header::SHN_XINDEX => {
                // The section header index overflowed 16 bits and we have to look it up from
                // the extended index table instead.  Each item in the SHT_SYMTAB_SHNDX section is
                // a 32-bit value even for a 64-bit ELF objects.
                let section_offset = symtab_shndx_section_offset
                    .ok_or_else(|| anyhow!("Symbol {name} has st_shndx=SHN_XINDEX, but no SHT_SYMTAB_SHNDX section present"))?;

                let offset = section_offset + (i * 4);
                let slice = file_data.get(offset..offset + 4).ok_or_else(|| {
                    anyhow!("Index error looking up {name} in the SHT_SYMTAB_SHNDX section")
                })?;
                // If the last statement succeeded, the slice is exactly 4 bytes, so this try_into
                // will never fail
                let byte_array = slice.try_into().unwrap();
                if elf.little_endian {
                    u32::from_le_bytes(byte_array) as usize
                } else {
                    u32::from_be_bytes(byte_array) as usize
                }
            }
            // The normal case is that we can just use `st_shndx`
            _ => sym.st_shndx,
        };

        let sh = elf
            .section_headers
            .get(header_index)
            .ok_or_else(|| anyhow!("Index error looking up section header for {name}"))?;

        // Offset relative to the start of the section.
        let section_offset = sym.st_value - sh.sh_addr;
        extracted.extract_item(name, file_data, (sh.sh_offset + section_offset) as usize)?;
    }
    Ok(extracted.into_metadata())
}

pub fn extract_from_pe(pe: PE<'_>, file_data: &[u8]) -> anyhow::Result<Vec<Metadata>> {
    let mut extracted = ExtractedItems::new();
    for export in pe.exports {
        if let Some(name) = export.name {
            if is_metadata_symbol(name) {
                extracted.extract_item(
                    name,
                    file_data,
                    export.offset.context("Error getting symbol offset")?,
                )?;
            }
        }
    }
    Ok(extracted.into_metadata())
}

pub fn extract_from_mach(mach: Mach<'_>, file_data: &[u8]) -> anyhow::Result<Vec<Metadata>> {
    match mach {
        Mach::Binary(macho) => extract_from_macho(macho, file_data),
        // Multi-binary library, just extract the first one
        Mach::Fat(multi_arch) => match multi_arch.get(0)? {
            SingleArch::MachO(macho) => extract_from_macho(macho, file_data),
            SingleArch::Archive(archive) => extract_from_archive(archive, file_data),
        },
    }
}

pub fn extract_from_macho(macho: MachO<'_>, file_data: &[u8]) -> anyhow::Result<Vec<Metadata>> {
    let mut sections: Vec<Section> = Vec::new();
    for sects in macho.segments.sections() {
        sections.extend(sects.map(|r| r.expect("section").0));
    }
    let mut extracted = ExtractedItems::new();
    sections.sort_by_key(|s| s.addr);

    // Iterate through the symbols.  This picks up symbols from the .o files embedded in a Darwin
    // archive.
    for (name, nlist) in macho.symbols().flatten() {
        // Check that the symbol:
        //   - Is global (exported)
        //   - Has type=N_SECT (it's regular data as opposed to something like
        //     "undefined" or "indirect")
        //   - Has a metadata symbol name
        if nlist.is_global()
            && nlist.get_type() == symbols::N_SECT
            && is_metadata_symbol(name)
            && nlist.n_sect != symbols::NO_SECT as usize
        {
            let section = &sections[nlist.n_sect - 1];

            // `nlist.n_value` is an address, so we can calculating the offset inside the section
            // using the difference between that and `section.addr`
            let offset = section.offset as usize + nlist.n_value as usize - section.addr as usize;
            extracted.extract_item(name, file_data, offset)?;
        }
    }

    // Iterate through the exports.  This picks up symbols from .dylib files.
    for export in macho.exports()? {
        let name = &export.name;
        if is_metadata_symbol(name) {
            extracted.extract_item(name, file_data, export.offset as usize)?;
        }
    }
    Ok(extracted.into_metadata())
}

pub fn extract_from_archive(
    archive: Archive<'_>,
    file_data: &[u8],
) -> anyhow::Result<Vec<Metadata>> {
    // Store the names of archive members that have metadata symbols in them
    let mut members_to_check: HashSet<&str> = HashSet::new();
    for (member_name, _, symbols) in archive.summarize() {
        for name in symbols {
            if is_metadata_symbol(name) {
                members_to_check.insert(member_name);
            }
        }
    }

    let mut items = vec![];
    for member_name in members_to_check {
        items.append(
            &mut extract_from_bytes(
                archive
                    .extract(member_name, file_data)
                    .with_context(|| format!("Failed to extract archive member `{member_name}`"))?,
            )
            .with_context(|| {
                format!("Failed to extract data from archive member `{member_name}`")
            })?,
        );
    }
    Ok(items)
}

pub fn extract_from_coff(coff: Coff<'_>, file_data: &[u8]) -> anyhow::Result<Vec<Metadata>> {
    // https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#coff-file-header-object-and-image
    let Some(strings) = coff.strings else {
        return Ok(vec![]);
    };
    let Some(symbols) = coff.symbols else {
        return Ok(vec![]);
    };

    let mut extracted = ExtractedItems::new();

    for (_, _, symbol) in symbols.iter() {
        // COFF uses a one-based section index. Non-positive values have specifial meanings.
        // 0 : IMAGE_SYM_UNDEFINED
        // -1: IMAGE_SYM_ABSOLUTE
        // -2: IMAGE_SYM_DEBUG
        // See https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#section-number-values
        // for details.
        if symbol.section_number <= 0 {
            continue;
        }
        let Ok(name) = symbol.name(&strings) else {
            continue;
        };
        let section = coff
            .sections
            .get((symbol.section_number - 1) as usize)
            .with_context(|| format!("Index error looking up section header for {name}"))?;
        if !is_metadata_symbol(name) {
            continue;
        }
        extracted.extract_item(
            name,
            file_data,
            (section.pointer_to_raw_data as usize)
                .checked_add(symbol.value as usize)
                .context("Error getting symbol offset")?,
        )?;
    }

    Ok(extracted.into_metadata())
}

pub fn extract_from_wasm(module: &Module) -> anyhow::Result<Vec<Metadata>> {
    fn get_global_positive_usize(module: &Module, global_id: GlobalId) -> Option<usize> {
        let global = module.globals.get(global_id);
        let GlobalKind::Local(const_expr) = &global.kind else {
            return None;
        };
        get_const_expr_positive_usize(module, const_expr)
    }

    fn get_const_expr_positive_usize(module: &Module, const_expr: &ConstExpr) -> Option<usize> {
        match const_expr {
            ConstExpr::Value(value) => match value {
                Value::I32(i32) => usize::try_from(*i32).ok(),
                Value::I64(i64) => usize::try_from(*i64).ok(),
                _ => None,
            },
            ConstExpr::Global(id) => get_global_positive_usize(module, *id),
            _ => None,
        }
    }

    let mut extracted = ExtractedItems::new();

    for export in module.exports.iter() {
        if !is_metadata_symbol(&export.name) {
            continue;
        }
        let ExportItem::Global(global_id) = &export.item else {
            continue;
        };
        // The exported global contains a pointer value to the data in memory 0.
        let Some(address) = get_global_positive_usize(module, *global_id) else {
            continue;
        };
        // An active data segment contains a memory ID (0 if not specified) and a byte array.
        // When the WASM module is initialized, the designated range of the memory is initialized
        // with this byte array data.
        for data in module.data.iter() {
            // A WASM module built with Rust usually stores global variable data in memory 0;
            // i.e., given a Rust module, `memory` should be 0 here.
            let DataKind::Active {
                offset: data_offset,
                // memory,
                ..
            } = &data.kind
            else {
                continue;
            };
            let Some(data_offset) = get_const_expr_positive_usize(module, data_offset) else {
                continue;
            };
            if !(data_offset..data_offset + data.value.len()).contains(&address) {
                continue;
            }
            // We have found the data segment that contains the UniFFI metadata.
            extracted.extract_item(
                &export.name,
                &data.value,
                address
                    .checked_sub(data_offset)
                    .context("Error getting global data offset")?,
            )?;
        }
    }

    Ok(extracted.into_metadata())
}

/// Container for extracted metadata items
#[derive(Default)]
struct ExtractedItems {
    items: Vec<Metadata>,
    /// symbol names for the extracted items, we use this to ensure that we don't extract the same
    /// symbol twice
    names: HashSet<String>,
}

impl ExtractedItems {
    fn new() -> Self {
        Self::default()
    }

    fn extract_item(&mut self, name: &str, file_data: &[u8], offset: usize) -> anyhow::Result<()> {
        if self.names.contains(name) {
            // Already extracted this item
            return Ok(());
        }

        // Use the file data starting from offset, without specifying the end position.  We don't
        // always know the end position, because goblin reports the symbol size as 0 for PE and
        // MachO files.
        //
        // This works fine, because `MetadataReader` knows when the serialized data is terminated
        // and will just ignore the trailing data.
        let data = &file_data[offset..];
        self.items.push(
            Metadata::read(data).with_context(|| format!("extracting metadata for '{name}'"))?,
        );
        self.names.insert(name.to_string());
        Ok(())
    }

    fn into_metadata(self) -> Vec<Metadata> {
        self.items
    }
}

fn is_metadata_symbol(name: &str) -> bool {
    // Skip the "_" char that Darwin prepends, if present
    let name = name.strip_prefix('_').unwrap_or(name);
    name.starts_with("UNIFFI_META")
}
