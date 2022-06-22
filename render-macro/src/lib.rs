//! Crate with macro implementations of `expression_format`.
//!
//! A separete crate is required to test procedural macros.

extern crate proc_macro;

use itertools::Itertools;
use proc_macro::TokenStream;
use regex::Captures;
use syn::{parse_macro_input, ExprLit, Lit};

// Taken from the once-cell docs
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[proc_macro]
pub fn render(tokens: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(tokens as ExprLit);
    match expr.lit {
        Lit::Str(s) => format_render(&s.value()).parse().unwrap(),
        _ => panic!("String literal expected"),
    }
}

#[proc_macro]
pub fn render_indoc(tokens: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(tokens as ExprLit);
    match expr.lit {
        Lit::Str(s) => format_render_indoc(&s.value()).parse().unwrap(),
        _ => panic!("String literal expected"),
    }
}

enum FormatArg {
    Plain(String),
    CommaJoin(String),
}

fn format_render(render_string: &str) -> String {
    // Start by quoting all `{`, `}`, and `"` chars
    let quoted_string = render_string
        .replace('{', "{{")
        .replace('}', "}}")
        .replace('"', r#"\""#);
    // Handle named parameters
    let named_param = regex!(r"\{\{([[:alpha:]][[:alnum:]\._]*)(,?)}}");
    let mut args = vec![];
    let format_string = named_param.replace_all(&quoted_string, |caps: &Captures| {
        let name = caps[1].to_string();
        args.push(match &caps[2] {
            "" => FormatArg::Plain(name),
            "," => FormatArg::CommaJoin(name),
            _ => unreachable!(),
        });
        "{}"
    });
    let format_args: String = args
        .iter()
        .map(|arg| match arg {
            FormatArg::Plain(name) => format!(", self.render(&{name})"),
            FormatArg::CommaJoin(name) => {
                format!(r#", (&{name}).into_iter().map(|i| self.render(i)).collect::<Vec<_>>().join(", ")"#)
            }
        })
        .collect();
    format!(r#"format!("{format_string}"{format_args})"#)
}

fn format_render_indoc(render_string: &str) -> String {
    let mut lines = render_string.lines().peekable();
    // Skip over the first newline
    if matches!(lines.peek(), Some(l) if l.is_empty()) {
        lines.next();
    }

    let min_leading_whitespace = lines
        .clone()
        .map(count_leading_whitespace)
        .filter(|v| *v > 0)
        .min();
    let to_strip = match min_leading_whitespace {
        None => return format_render(""),
        Some(value) => value,
    };

    format_render(
        &lines
            .map(|line| {
                if line.is_empty() {
                    line
                } else {
                    &line[to_strip..]
                }
            })
            .join("\n"),
    )
}

fn count_leading_whitespace(value: &str) -> usize {
    value.chars().take_while(|c| *c == ' ').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        assert_eq!(format_render("hello world"), r#"format!("hello world")"#);
        assert_eq!(
            format_render("hello {world}"),
            r#"format!("hello {}", self.render(&world))"#
        );
        assert_eq!(
            format_render("{a} == {b}"),
            r#"format!("{} == {}", self.render(&a), self.render(&b))"#
        );
        assert_eq!(
            format_render("{hello.world}"),
            r#"format!("{}", self.render(&hello.world))"#
        );
        assert_eq!(
            format_render("{ {a} == {b} }"),
            r#"format!("{{ {} == {} }}", self.render(&a), self.render(&b))"#
        );
        assert_eq!(
            format_render("\"{foo}\""),
            r#"format!("\"{}\"", self.render(&foo))"#
        );
    }

    #[test]
    fn test_comma_join() {
        assert_eq!(
            format_render("\"{foo,}\""),
            r#"format!("\"{}\"", (&foo).into_iter().map(|i| self.render(i)).collect::<Vec<_>>().join(", "))"#
        );
    }

    #[test]
    fn test_render_indoc() {
        assert_eq!(
            format_render_indoc("  hello\n  world"),
            "format!(\"hello\nworld\")"
        );
        assert_eq!(
            format_render_indoc("  hello\n    world"),
            "format!(\"hello\n  world\")"
        );
        assert_eq!(
            format_render_indoc("\n  hello\n    world"),
            "format!(\"hello\n  world\")"
        );
    }
}
