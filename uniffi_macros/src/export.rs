use proc_macro2::TokenStream;

mod metadata;
mod scaffolding;

use self::metadata::gen_metadata;
use self::scaffolding::gen_scaffolding;

// TODO(jplatte): Ensure no generics, no async, …
// TODO(jplatte): Aggregate errors instead of short-circuiting, whereever possible

enum ExportItem {
    Function {
        item: syn::ItemFn,
        checksum: u16,
        tracked_file: TokenStream,
    },
}

pub fn expand_export(item: syn::Item, mod_path: &[String]) -> syn::Result<TokenStream> {
    let item = gen_metadata(item, mod_path)?;
    gen_scaffolding(item, mod_path)
}
