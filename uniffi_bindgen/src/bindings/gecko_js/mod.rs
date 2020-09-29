/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub mod gen_gecko_js;
mod webidl;
pub use gen_gecko_js::{
    Config, Interface, InterfaceHeader, Namespace, NamespaceHeader, SharedHeader, WebIdl,
};

use super::super::interface::ComponentInterface;

pub struct Source {
    name: String,
    header: String,
    source: String,
}

pub struct Bindings {
    webidl: String,
    shared_header: String,
    sources: Vec<Source>,
}

/// Generate uniffi component bindings for Firefox.
///
/// Firefox's WebIDL binding declarations, generated by `Codegen.py` in m-c,
/// expect to find a `.h`/`.cpp` pair per interface, even if those interfaces
/// are declared in a single WebIDL file. Dictionaries and enums are
/// autogenerated by `Codegen.py`, so we don't need to worry about them...but
/// we do need to emit serialization code for them, plus the actual interface
/// and top-level function implementations, in the UniFFI bindings.
///
/// So the Gecko backend generates:
///
/// * A single WebIDL file with the component interface. This is similar to the
///   UniFFI IDL format, but the names of some types are different.
/// * A shared C++ header, with serialization helpers for all built-in and
///   interface types.
/// * A header and source file for the namespace, if the component defines any
///   top-level functions.
/// * A header and source file for each `interface` declaration in the UniFFI.
///   IDL.
///
/// These files should be checked in to the Firefox source tree. The WebIDL
/// file goes in `dom/chrome-webidl`, and the header and source files can be
/// added to any directory and referenced in `moz.build`. The Rust component
/// library must also be added as a dependency to `gkrust-shared` (in
/// `toolkit/library/rust/shared`), so that the FFI symbols are linked into
/// libxul.
pub fn write_bindings(
    config: &Config,
    ci: &ComponentInterface,
    out_dir: &Path,
    _try_format_code: bool,
) -> Result<()> {
    use heck::CamelCase;

    let out_path = PathBuf::from(out_dir);

    let Bindings {
        webidl,
        shared_header,
        sources,
    } = generate_bindings(config, ci)?;

    let mut webidl_file = out_path.clone();
    webidl_file.push(format!("{}.webidl", namespace_to_file_name(ci.namespace())));
    let mut w = File::create(&webidl_file).context("Failed to create WebIDL file for bindings")?;
    write!(w, "{}", webidl)?;

    let mut shared_header_file = out_path.clone();
    shared_header_file.push(format!(
        "{}Shared.h",
        namespace_to_file_name(ci.namespace())
    ));
    let mut h = File::create(&shared_header_file)
        .context("Failed to create shared header file for bindings")?;
    write!(h, "{}", shared_header)?;

    for Source {
        name,
        header,
        source,
    } in sources
    {
        let mut header_file = out_path.clone();
        header_file.push(format!("{}.h", namespace_to_file_name(&name)));
        let mut h = File::create(&header_file)
            .with_context(|| format!("Failed to create header file for `{}` bindings", name))?;
        write!(h, "{}", header)?;

        let mut source_file = out_path.clone();
        source_file.push(format!("{}.cpp", namespace_to_file_name(&name)));
        let mut w = File::create(&source_file)
            .with_context(|| format!("Failed to create header file for `{}` bindings", name))?;
        write!(w, "{}", source)?;
    }

    Ok(())
}

pub fn namespace_to_file_name(namespace: &str) -> String {
    use heck::CamelCase;
    namespace.to_camel_case()
}

/// Generate Gecko bindings for the given ComponentInterface, as a string.
pub fn generate_bindings(config: &Config, ci: &ComponentInterface) -> Result<Bindings> {
    use askama::Template;
    use heck::CamelCase;

    let webidl = WebIdl::new(config, ci)
        .render()
        .context("Failed to render WebIDL bindings")?;

    let shared_header = SharedHeader::new(config, ci)
        .render()
        .context("Failed to render shared header")?;

    let mut sources = Vec::new();

    // Top-level functions go in one namespace, which needs its own header and
    // source file.
    let functions = ci.iter_function_definitions();
    if !functions.is_empty() {
        let context = gen_gecko_js::Context::new(config, ci);
        let header = NamespaceHeader::new(context, functions.as_slice())
            .render()
            .context("Failed to render top-level namespace header")?;
        let source = Namespace::new(context, functions.as_slice())
            .render()
            .context("Failed to render top-level namespace binding")?;
        sources.push(Source {
            name: ci.namespace().into(),
            header,
            source,
        });
    }

    // Now generate one header/source pair for each interface.
    let objects = ci.iter_object_definitions();
    for obj in objects {
        let context = gen_gecko_js::Context::new(config, ci);
        let header = InterfaceHeader::new(context, &obj)
            .render()
            .with_context(|| format!("Failed to render {} header", obj.name()))?;
        let source = Interface::new(context, &obj)
            .render()
            .with_context(|| format!("Failed to render {} binding", obj.name()))?;
        sources.push(Source {
            name: obj.name().into(),
            header,
            source,
        });
    }

    Ok(Bindings {
        webidl,
        shared_header,
        sources,
    })
}
