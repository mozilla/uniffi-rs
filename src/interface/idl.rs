/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module implements a parser for the uniffi Interface Definition Language.
//!
//! The syntax is very heavily based on [WebIDL](https://heycam.github.io/webidl/)
//! and [OMG IDL](https://www.omg.org/spec/IDL/About-IDL/), but with some adaptations
//! to make it more comfortably fit out needs.
//!
//! Here is a small-but-complete example to give a flavour of the language:
//!
//! ```
//! # let defn = uniffi::interface::idl::ModuleDefinition::parse(r##"
//! #
//! /* All functionality is encapsulated in a named "module" */
//!
//! module example {
//!
//!     // You can define basic C-style enums.
//!     // For now richer Rust-style enums are not supported.
//!
//!     enum Color {
//!         RED,
//!         GREEN,
//!         BLUE
//!     };
//!
//!     // You can define compound data structures using "records", which have typed named
//!     // fields and which get passes around by value.
//!     // This one represents a point in two-dimensional space.
//!
//!     record Point {
//!       float x;
//!       float y;
//!     };
//!
//!     // You can specify plain functions for functionality offered by the module.
//!     // This one accepts a list of `Point` records and returns the one that is farther
//!     // from the origin, or `null` if the list of points is empty.
//!
//!     Point? farthest_from_origin(sequence<Point> points);
//!
//!     // You can specify classes that encapsulate both state and behaviour.
//!     // This one is an object that can be moved around in the world and change
//!     // colour over time.
//!
//!     class Sprite {
//!         // Classes may have a constructor
//!         // (and they're currently restricted to at most one constructor).
//!
//!         constructor(Point position, Colour colour);
//!
//!         // Currently, classes can *only* expose functionality via methods,
//!         // they cannot directly expose data fields.
//!         // This might change in future, but it's complicated.
//!
//!         Point get_position();
//!         Color get_colour();
//!
//!         void move_to(Point new_position);
//!         void set_colour(Colour new_colour);
//!     };
//! };
//! # "##)?;
//! # assert_eq!(defn.name, "example");
//! # Ok::<(), anyhow::Error>(())
//! ```
//! Consumers of this module would parse a string containing such an IDL specification
//! using [`ModuleDefinition::parse`], then inspect the resulting tree of structs.
//!
//! The syntax is documented in detail on the various types that implement the parser,
//! but can be summarized as follows:
//!
//! * The underlying syntax of IDL consists of [keywords](kwd), [symbols](sym), and
//!   [identifiers](identifier), all potentially separated by ignoreable [padding]
//!   such as whitespace and comments.
//! * The functionality defined by an IDL document operates on data of various named
//!   [types](TypeReference), which include the usual primitives (booleans, floats,
//!   strings, integers of various size and signedness) as well as compound types
//!   defined using the keywords below.
//! * At the top level of the file we expect a single [`module`](ModuleDefinition)
//!   definition, which provides metadata about the component as a whole and serves
//!   as a namespace for the functionality to be exported.
//! * The [`enum`](EnumDefinition) keyword defines a C-style enum as a new named type,
//!   which can take on one of a fixed set of named values.
//! * The [`record`](RecordDefinition) keyword defines a compound data type made up
//!   of one or more named [fields](FieldDefinition), each with a particular type.
//! * The [`class`](ClassDefinition) keyword defines a new opaque data type that
//!   encapsulates state and behvaiour, and provides named [methods](MethodDefinition)
//!   that can be invoked on instances of that class.
//! * A module can also contain top-level [function](FunctionDefinition) definitions,
//!   which can be invoked directly without requiring a class or instance.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1, take_while_m_n},
    character::complete::{anychar, multispace1, newline},
    combinator::{map, opt, peek},
    multi::{many0, many_till, separated_list},
    sequence::{delimited, preceded, terminated, tuple},
};

/// A custom `nom` result type that fixes some details of how this parser works.
///
/// We only ever work with &str inputs in this file, and we only ever expect to generate
/// basic errors. By fixing this types in the signature of all our functions we make it
/// easier for the rust typechecker to handle the various combinators (and hopefully,
/// easier for our future selves to update to more expressive error handling).
type ParseResult<'src, O> = nom::IResult<&'src str, O, (&'src str, nom::error::ErrorKind)>;

/// Every uniffi IDL file must contain a single top-level `module` definition, which provides
/// metadata for the component as a whole and which contains all the functionality exported
/// by the component:
///
/// ```text
/// module example {
///    /* interfaces, value types, enums etc declared here */
/// }
/// ```
pub struct ModuleDefinition<'src> {
    pub name: &'src str,
    members: Vec<ModuleMember<'src>>,
}

impl<'src> ModuleDefinition<'src> {
    /// Parses a [`ModuleDefinition`] from an IDL source string.
    ///
    /// This is expected to be the main entrypoint to this module for consumers. It does a full parse of the given
    /// input string and returns an Error if it contains any unparseable data. The returned [`ModuleDefinition`]
    /// is bounded by the lifetime of the input string because it will contain many borrowed references to snippets
    /// from therein.
    pub fn parse(input: &'src str) -> anyhow::Result<Self> {
        let (rest, module) = Self::parse_from(input)
            .map_err(|e| anyhow::anyhow!("Failed to parse module: {:?}", e))?;
        if rest != "" {
            anyhow::bail!("Input contains unparseable data: {:?}", rest)
        }
        Ok(module)
    }

    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((
                kwd("module"),
                identifier,
                sym("{"),
                many0(ModuleMember::parse_from),
                sym("}"),
                sym(";"),
            )),
            |(_, name, _, members, _, _)| ModuleDefinition { name, members },
        )(input)
    }
}

/// Parses any of the items that can be contained in a module:
/// [`EnumDefinition`], [`RecordDefinition`], [`ObjectDefinition`] or [`FunctionDefinition`].
pub(crate) enum ModuleMember<'src> {
    Enum(EnumDefinition<'src>),
    Record(RecordDefinition<'src>),
    Class(ClassDefinition<'src>),
    Function(FunctionDefinition<'src>),
}

impl<'src> ModuleMember<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        alt((
            map(EnumDefinition::parse_from, |defn| Self::Enum(defn)),
            map(RecordDefinition::parse_from, |defn| Self::Record(defn)),
            map(ClassDefinition::parse_from, |defn| Self::Class(defn)),
            map(FunctionDefinition::parse_from, |defn| Self::Function(defn)),
        ))(input)
    }
}

/// The [`enum`](EnumDefinition) keyword defines a C-style enum as a new named type.
/// Each enum has a fixed number of options which can be specified by name. It is not
/// possible to set or to example the internal representation of the enum.
///
/// ```text
/// enum exampleNameOfEnum {
///      /* An identifier for each option, separated by commas */
///     OPTION_ONE,
///     OPTION_TWO,
/// }
/// ```
pub(crate) struct EnumDefinition<'src> {
    name: &'src str,
    members: Vec<&'src str>,
}

impl<'src> EnumDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((
                kwd("enum"),
                identifier,
                sym("{"),
                comma_separated_list(identifier),
                sym("}"),
                sym(";"),
            )),
            |(_, name, _, members, _, _)| EnumDefinition { name, members },
        )(input)
    }
}

pub(crate) struct RecordDefinition<'src> {
    name: &'src str,
    fields: Vec<FieldDefinition<'src>>,
}

impl<'src> RecordDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((
                kwd("record"),
                identifier,
                sym("{"),
                many0(FieldDefinition::parse_from),
                sym("}"),
                sym(";"),
            )),
            |(_, name, _, fields, _, _)| RecordDefinition { name, fields },
        )(input)
    }
}

pub(crate) struct FieldDefinition<'src> {
    name: &'src str,
    type_: TypeReference<'src>,
}

impl<'src> FieldDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((TypeReference::parse_from, identifier, sym(";"))),
            |(type_, name, _)| FieldDefinition { name, type_ },
        )(input)
    }
}

pub(crate) enum TypeReference<'src> {
    Boolean,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Float,
    Double,
    Nullable(Box<TypeReference<'src>>),
    Sequence(Box<TypeReference<'src>>),
    Named(&'src str),
}

impl<'src> TypeReference<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        let (input, type_) = alt((
            map(kwd("boolean"), |_| TypeReference::Boolean),
            map(kwd("u8"), |_| TypeReference::U8),
            map(kwd("i8"), |_| TypeReference::I8),
            map(kwd("u16"), |_| TypeReference::U16),
            map(kwd("i16"), |_| TypeReference::I16),
            map(kwd("u32"), |_| TypeReference::U32),
            map(kwd("i32"), |_| TypeReference::I32),
            map(kwd("u64"), |_| TypeReference::U64),
            map(kwd("i64"), |_| TypeReference::I64),
            map(kwd("float"), |_| TypeReference::Float),
            map(kwd("double"), |_| TypeReference::Double),
            map(
                tuple((
                    kwd("sequence"),
                    sym("<"),
                    TypeReference::parse_from,
                    sym(">"),
                )),
                |(_, _, type_, _)| TypeReference::Sequence(Box::new(type_)),
            ),
            map(identifier, |name| TypeReference::Named(name)),
        ))(input)?;
        // Any of the above types can be made nullable by appending a `?`,
        // but nullability itself doesn't nest (so you can't have e.g. `int??`).
        let (input, nullable) = opt(sym("?"))(input)?;
        let type_ = match nullable {
            None => type_,
            Some(_) => TypeReference::Nullable(Box::new(type_)),
        };
        Ok((input, type_))
    }
}

pub(crate) struct FunctionDefinition<'src> {
    name: &'src str,
    arguments: Vec<ArgumentDefinition<'src>>,
    return_type: ReturnTypeDefinition<'src>,
}

impl<'src> FunctionDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((
                ReturnTypeDefinition::parse_from,
                identifier,
                sym("("),
                comma_separated_list(ArgumentDefinition::parse_from),
                sym(")"),
                sym(";"),
            )),
            |(return_type, name, _, arguments, _, _)| FunctionDefinition {
                name,
                arguments,
                return_type,
            },
        )(input)
    }
}

pub(crate) struct ArgumentDefinition<'src> {
    name: &'src str,
    type_: TypeReference<'src>,
}

impl<'src> ArgumentDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((TypeReference::parse_from, identifier)),
            |(type_, name)| ArgumentDefinition { name, type_ },
        )(input)
    }
}

pub(crate) struct ReturnTypeDefinition<'src> {
    return_type: Option<TypeReference<'src>>,
}

impl<'src> ReturnTypeDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            alt((
                map(kwd("void"),|_| None),
                map(TypeReference::parse_from, |type_| Some(type_)),
            )),
            |(return_type)| ReturnTypeDefinition {
                return_type,
            },
        )(input)
    }
}


pub(crate) struct ClassDefinition<'src> {
    name: &'src str,
    members: Vec<ClassMember<'src>>,
}

impl<'src> ClassDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((
                kwd("class"),
                identifier,
                sym("{"),
                many0(ClassMember::parse_from),
                sym("}"),
                sym(";"),
            )),
            |(_, name, _, members, _, _)| ClassDefinition { name, members },
        )(input)
    }
}

/// Parses any of the items that can be contained in a class:
/// [`ConstructorDefinition`], [`MethodDefinition`].
pub(crate) enum ClassMember<'src> {
    Constructor(ConstructorDefinition<'src>),
    Method(MethodDefinition<'src>),
}

impl<'src> ClassMember<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        alt((
            map(ConstructorDefinition::parse_from, |defn| {
                Self::Constructor(defn)
            }),
            map(MethodDefinition::parse_from, |defn| Self::Method(defn)),
        ))(input)
    }
}

pub(crate) struct ConstructorDefinition<'src> {
    arguments: Vec<ArgumentDefinition<'src>>,
}

impl<'src> ConstructorDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((
                kwd("constructor"),
                sym("("),
                comma_separated_list(ArgumentDefinition::parse_from),
                sym(")"),
                sym(";"),
            )),
            |(_, _, arguments, _, _)| ConstructorDefinition { arguments },
        )(input)
    }
}

pub(crate) struct MethodDefinition<'src> {
    name: &'src str,
    arguments: Vec<ArgumentDefinition<'src>>,
    return_type: TypeReference<'src>,
}

impl<'src> MethodDefinition<'src> {
    fn parse_from(input: &'src str) -> ParseResult<'src, Self> {
        map(
            tuple((
                TypeReference::parse_from,
                identifier,
                sym("("),
                comma_separated_list(ArgumentDefinition::parse_from),
                sym(")"),
                sym(";"),
            )),
            |(return_type, name, _, arguments, _, _)| MethodDefinition {
                name,
                arguments,
                return_type,
            },
        )(input)
    }
}

/// Parses a comma-separated list of items from a given parser.
///
/// With optional trailing comma, because optional trailing commas are cool.
fn comma_separated_list<'src, F, O>(func: F) -> impl Fn(&'src str) -> ParseResult<'src, Vec<O>>
where
    F: Fn(&'src str) -> ParseResult<'src, O>,
{
    terminated(separated_list(sym(","), func), opt(sym(",")))
}

/// Parses ignoreable padding (whitespace, comments, etc).
///
/// This deliberately does not capture of any of the padding, because we don't need it
/// and because it makes the types much simpler.
fn padding<'src>(input: &'src str) -> ParseResult<()> {
    // We `map` to () so we don't capture any of the padding.
    // We `many0` for zero or more of any of these alternatives.
    map(
        many0(alt((
            map(multispace1, |_| ()),
            map(preceded(tag("//"), many_till(anychar, newline)), |_| ()),
            map(preceded(tag("/*"), many_till(anychar, tag("*/"))), |_| ()),
        ))),
        |_| (),
    )(input)
}

/// Parses a keyword: a fixed alphanumeric string followed by a non-identifier character.
fn kwd<'src>(name: &'static str) -> impl Fn(&'src str) -> ParseResult<'src, &'src str> {
    delimited(
        padding,
        // This is "followed by one non-identifier character", using lookahead.
        terminated(
            tag(name),
            peek(take_while_m_n(1, 1, |c: char| !is_identifier_char(c))),
        ),
        padding,
    )
}

/// Parses a symbol: a fixed non-alphanumeric string.
fn sym<'src>(symbol: &'static str) -> impl Fn(&'src str) -> ParseResult<'src, &'src str> {
    delimited(padding, tag(symbol), padding)
}

/// Check whether a character can be part of an identifier.
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Parses an identifier: a sequence of characters from [A-Za-z0-9_].
///
/// XXX TODO: should not be allowed to start with a number, or match a keyword.
fn identifier<'src>(input: &'src str) -> ParseResult<'src, &'src str> {
    delimited(
        padding,
        // XXX TODO: nom probably has a better helper for this, with a regex or something...
        take_while1(is_identifier_char),
        padding,
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;
    type TestResult = anyhow::Result<()>;

    /// A helper macro for writing tests about parsing errors.
    /// This simplifies both asserting that the parse failed, and extracting the details
    /// of the failure so we can assert things aboout it.
    macro_rules! get_parse_error {
        ($e:expr) => {
            match $e {
                Err(nom::Err::Error((rest, err))) => (rest, err),
                _ => anyhow::bail!("parser did not return an error"),
            }
        };
    }

    #[test]
    fn test_keyword() -> TestResult {
        let (rest, matched) = kwd("test")("test me")?;
        assert_eq!(matched, "test");
        assert_eq!(rest, "me");

        let (rest, matched) = kwd("test")(" \ttest \r\nme")?;
        assert_eq!(matched, "test");
        assert_eq!(rest, "me");

        let (rest, _) = get_parse_error!(kwd("test")("not this test"));
        assert_eq!(rest, "not this test");

        Ok(())
    }

    #[test]
    fn test_keyword_does_not_accidentally_prefix_match() -> TestResult {
        let (rest, _) = get_parse_error!(kwd("test")("testeth me"));
        assert_eq!(rest, "eth me");
        Ok(())
    }

    #[test]
    fn test_symbol() -> TestResult {
        let (rest, matched) = sym("?")("?")?;
        assert_eq!(matched, "?");
        assert_eq!(rest, "");

        let (rest, matched) = sym("?")(" ? \r\n foo")?;
        assert_eq!(matched, "?");
        assert_eq!(rest, "foo");

        let (rest, matched) = sym("?")(" ?;")?;
        assert_eq!(matched, "?");
        assert_eq!(rest, ";");

        let (rest, _) = get_parse_error!(sym("?")("!?"));
        assert_eq!(rest, "!?");

        Ok(())
    }

    #[test]
    fn test_identifier() -> TestResult {
        let (rest, matched) = identifier("hello_world")?;
        assert_eq!(matched, "hello_world");
        assert_eq!(rest, "");

        let (rest, matched) = identifier(" \thello world")?;
        assert_eq!(matched, "hello");
        assert_eq!(rest, "world");

        // XXX TODO: probably it shouldn't start with a number...
        let (rest, _) = get_parse_error!(identifier("!name"));
        assert_eq!(rest, "!name");

        Ok(())
    }

    #[test]
    fn test_parse_empty_enum() -> TestResult {
        let (rest, defn) = EnumDefinition::parse_from("enum testEnum {};")?;
        assert_eq!(rest, "");
        assert_eq!(defn.name, "testEnum");
        Ok(())
    }
    #[test]
    fn test_parse_basic_enum() -> TestResult {
        let (rest, defn) = EnumDefinition::parse_from("enum testEnum { ONE, TWO, THREE };")?;
        assert_eq!(rest, "");
        assert_eq!(defn.name, "testEnum");
        assert_eq!(defn.members, vec!["ONE", "TWO", "THREE"]);
        Ok(())
    }
}
