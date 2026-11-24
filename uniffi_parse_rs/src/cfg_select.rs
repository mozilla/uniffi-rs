/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code to handle the cfg_select! macro
//!
//! This also handles `cfg_if!` since that's still in wide use and it can reuse the same logic.

use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token::Brace,
    Macro, Meta, Token,
};

use crate::{
    kw, paths::LookupCache, BuiltinItem, CompileEnv, Error, Ir, IrChanges, Namespace, Result,
    VisitUnresolvedItem,
};

pub fn resolve_cfg_select_macros(ir: &mut Ir) -> Result<()> {
    // Use a loop, since each time we expand one of these macros it might lead to more expansions
    loop {
        let mut expanded_macro = false;

        let mut changes = IrChanges::default();
        let mut cache = LookupCache::default();
        ir.try_visit_unresolved_items(|crate_root, module, path, unresolved| {
            let syn::Item::Macro(mac) = unresolved else {
                return Ok(VisitUnresolvedItem::Keep);
            };
            let Some(builtin) =
                path.resolve_builtin(ir, &mut cache, &mac.mac.path, Namespace::Macro)
            else {
                return Ok(VisitUnresolvedItem::Keep);
            };
            match builtin {
                BuiltinItem::CfgSelectMacro => {
                    changes.extend_syn_items(
                        module.id,
                        resolve_cfg_select(&crate_root.compile_env, &mac.mac)
                            .map_err(|e| Error::new_syn(module.source, e))?,
                    );
                    expanded_macro = true;
                    Ok(VisitUnresolvedItem::Remove)
                }
                BuiltinItem::CfgIfMacro => {
                    changes.extend_syn_items(
                        module.id,
                        resolve_cfg_if(&crate_root.compile_env, &mac.mac)
                            .map_err(|e| Error::new_syn(module.source, e))?,
                    );
                    expanded_macro = true;
                    Ok(VisitUnresolvedItem::Remove)
                }
                _ => Ok(VisitUnresolvedItem::Keep),
            }
        })?;
        changes.apply(ir)?;
        if !expanded_macro {
            break;
        }
    }
    Ok(())
}

fn resolve_cfg_select(env: &CompileEnv, mac: &Macro) -> syn::Result<Vec<syn::Item>> {
    let select = mac.parse_body_with(CfgSelect::parse)?;
    select.resolve(env)
}

fn resolve_cfg_if(env: &CompileEnv, mac: &Macro) -> syn::Result<Vec<syn::Item>> {
    let select = mac.parse_body_with(CfgSelect::parse_cfg_if)?;
    select.resolve(env)
}

/// Parsed `cfg_select!` body
#[derive(Default)]
pub struct CfgSelect {
    branches: Vec<CfgSelectBranch>,
    else_: Option<Vec<syn::Item>>,
}

pub struct CfgSelectBranch {
    condition: Meta,
    items: Vec<syn::Item>,
}

impl Parse for CfgSelect {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut parsed = Self::default();
        while !input.is_empty() {
            if parsed.else_.is_some() {
                return Err(syn::Error::new(input.span(), "expected macro end"));
            }
            if !input.peek(Token![_]) {
                parsed.branches.push(Self::parse_branch(input)?);
            } else {
                let _: Token![_] = input.parse()?;
                let _: Token![=>] = input.parse()?;
                parsed.else_ = Some(Self::parse_items(input)?);
            }
        }
        Ok(parsed)
    }
}

impl CfgSelect {
    fn resolve(self, env: &CompileEnv) -> syn::Result<Vec<syn::Item>> {
        for branch in self.branches {
            if env.check(&branch.condition)? {
                return Ok(branch.items);
            }
        }

        Ok(self.else_.unwrap_or_default())
    }

    fn parse_branch(input: ParseStream<'_>) -> syn::Result<CfgSelectBranch> {
        let condition = input.parse()?;
        let _: Token![=>] = input.parse()?;
        Ok(CfgSelectBranch {
            condition,
            items: Self::parse_items(input)?,
        })
    }

    fn parse_items(input: ParseStream<'_>) -> syn::Result<Vec<syn::Item>> {
        let content;
        braced!(content in input);
        let mut items = vec![];
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(items)
    }

    /// Parse a `cfg_if!` body and construct a `CfgSelect` from it
    fn parse_cfg_if(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut parsed = Self::default();
        let _: Token![if] = input.parse()?;
        parsed.branches.push(CfgSelectBranch {
            condition: Self::parse_cfg_if_condition(input)?,
            items: Self::parse_items(input)?,
        });
        while !input.is_empty() {
            if parsed.else_.is_some() {
                return Err(syn::Error::new(input.span(), "expected macro end"));
            }
            let _: Token![else] = input.parse()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![if]) {
                let _: Token![if] = input.parse()?;
                parsed.branches.push(CfgSelectBranch {
                    condition: Self::parse_cfg_if_condition(input)?,
                    items: Self::parse_items(input)?,
                });
            } else if lookahead.peek(Brace) {
                parsed.else_ = Some(Self::parse_items(input)?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(parsed)
    }

    fn parse_cfg_if_condition(input: ParseStream<'_>) -> syn::Result<Meta> {
        let _: Token![#] = input.parse()?;
        let content;
        bracketed!(content in input);
        let _: kw::cfg = content.parse()?;
        let content2;
        parenthesized!(content2 in content);
        content2.parse()
    }
}

#[cfg(test)]
mod test {
    use target_lexicon::Triple;

    use crate::{
        paths::{tests::run_resolve_item, LookupCache},
        ErrorKind, Ir,
    };

    use super::*;

    #[test]
    fn test_cfg_select() {
        // When feature-a is present, then RecordA is defined
        let ir = Ir::new_for_test_with_env(
            &["cfg_select_test"],
            CompileEnv::new(Triple::unknown(), vec!["feature-a".into()]),
        );
        let mut cache = LookupCache::default();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordA"),
            Ok("cfg_select_test::RecordA".into())
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordB"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordC"),
            Err(ErrorKind::NotFound),
        );

        // When feature-b is present, then RecordB is defined
        let ir = Ir::new_for_test_with_env(
            &["cfg_select_test"],
            CompileEnv::new(Triple::unknown(), vec!["feature-b".into()]),
        );
        let mut cache = LookupCache::default();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordA"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordB"),
            Ok("cfg_select_test::RecordB".into())
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordC"),
            Err(ErrorKind::NotFound),
        );

        // If both features are present, then feature-a wins
        let ir = Ir::new_for_test_with_env(
            &["cfg_select_test"],
            CompileEnv::new(
                Triple::unknown(),
                vec!["feature-a".into(), "feature-b".into()],
            ),
        );
        let mut cache = LookupCache::default();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordA"),
            Ok("cfg_select_test::RecordA".into())
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordB"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordC"),
            Err(ErrorKind::NotFound),
        );

        // If neither features are enabled, then record C is defined
        let ir = Ir::new_for_test_with_env(
            &["cfg_select_test"],
            CompileEnv::new(Triple::unknown(), vec![]),
        );
        let mut cache = LookupCache::default();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordA"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordB"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "RecordC"),
            Ok("cfg_select_test::RecordC".into())
        );
    }

    // Same as test_cfg_select, but using `cfg_if`
    #[test]
    fn test_cfg_if() {
        // When feature-a is present, then RecordA is defined
        let ir = Ir::new_for_test_with_env(
            &["cfg_if_test"],
            CompileEnv::new(Triple::unknown(), vec!["feature-a".into()]),
        );
        let mut cache = LookupCache::default();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordA"),
            Ok("cfg_if_test::RecordA".into())
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordB"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordC"),
            Err(ErrorKind::NotFound),
        );

        // When feature-b is present, then RecordB is defined
        let ir = Ir::new_for_test_with_env(
            &["cfg_if_test"],
            CompileEnv::new(Triple::unknown(), vec!["feature-b".into()]),
        );
        let mut cache = LookupCache::default();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordA"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordB"),
            Ok("cfg_if_test::RecordB".into())
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordC"),
            Err(ErrorKind::NotFound),
        );

        // If both features are present, then feature-a wins
        let ir = Ir::new_for_test_with_env(
            &["cfg_if_test"],
            CompileEnv::new(
                Triple::unknown(),
                vec!["feature-a".into(), "feature-b".into()],
            ),
        );
        let mut cache = LookupCache::default();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordA"),
            Ok("cfg_if_test::RecordA".into())
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordB"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordC"),
            Err(ErrorKind::NotFound),
        );

        // If neither features are enabled, then record C is defined
        let ir =
            Ir::new_for_test_with_env(&["cfg_if_test"], CompileEnv::new(Triple::unknown(), vec![]));
        let mut cache = LookupCache::default();
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordA"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordB"),
            Err(ErrorKind::NotFound),
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_if_test", "RecordC"),
            Ok("cfg_if_test::RecordC".into())
        );
    }

    /// Test tricky cases that require multiple item resolution passes to get right
    #[test]
    fn test_multi_pass_resolution() {
        let env = CompileEnv::new(Triple::unknown(), vec!["enabled-feature".into()]);
        let ir = Ir::new_for_test_with_env(&["cfg_select_test"], env);
        let mut cache = LookupCache::default();

        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test::mod1", "RecordA"),
            Ok("cfg_select_test::mod1::RecordA".into())
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "mod2::RecordB"),
            Ok("cfg_select_test::mod2::RecordB".into())
        );
        assert_eq!(
            run_resolve_item(&ir, &mut cache, "cfg_select_test", "mod2::RecordAAlias"),
            Ok("cfg_select_test::mod2::RecordAAlias".into())
        );
    }
}
