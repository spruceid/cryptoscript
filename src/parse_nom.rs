use crate::elem::Elem;

use std::sync::Arc;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while_m_n, take_till};
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1};
use nom::combinator::{map, map_opt, map_res, recognize, value, verify};
use nom::multi::{fold_many0, many0, many0_count, separated_list1};
use nom::sequence::{delimited, pair, preceded};
use nom::error::{FromExternalError, ParseError};
use nom::Parser;

// var = app // comment
//
// app := function(arg0, arg1, .., argN)
// arg := var | app | literal
// literal := Elem
// var := alpha[alpha | num | '_']+
// function := var // parsed as function contextually

/// Line comments: "//comment"
pub type Comment = String;

/// Rust-style variables
pub type Var = String;

/// A parsed source file
#[derive(Debug, Clone)]
pub struct SourceCode {
    /// Vec of SourceBlock's, in order
    blocks: Vec<SourceBlock>
}

/// A single block of parsed source code
#[derive(Debug, Clone, PartialEq)]
pub enum SourceBlock {
    /// A line comment
    Comment(Comment),

    /// An assignment, which could span multiple lines
    Assignment(Assignment),
}

/// A single assignment: assignments are "simple," i.e. no pattern matching, etc
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    /// Assigned variable
    var: Var,

    /// Expression assigned
    app: App,
}

/// An application of a function to a Vec of arguments
#[derive(Debug, Clone, PartialEq)]
pub struct App {
    /// The function variable: "f" in "f(1, 2, 3)"
    function: Var,

    /// The argument expressions: "[1, 2, 3]" in "f(1, 2, 3)"
    args: Vec<Expr>,
}

/// Parsed expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Function application
    App(App),

    /// Literal
    Lit(String),

    /// Variable
    Var(Var),
}

// variables may start with a letter (or underscore) and may contain underscores and alphanumeric characters
fn parse_var(input: &str) -> IResult<&str, Var> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))))
    )(input)
        .map(|(i, o)| (i, o.to_string()))
}

#[cfg(test)]
mod test_parse_var {
    use super::*;

    #[test]
    fn test_single_char_var() {
        for s in (b'a' ..= b'z').map(|x| char::to_string(&char::from(x))) {
            assert_eq!(parse_var(&s), Ok(("", s.clone())))
        }
    }
}

fn parse_comment(input: &str) -> IResult<&str, SourceBlock> {
    preceded(tag("//"), take_till(|c| c == '\r' || c == '\n'))(input)
        .map(|(i, o)| (i, SourceBlock::Comment(o.to_string())))
}

#[cfg(test)]
mod test_parse_comment {
    use super::*;

    #[test]
    fn test_single_char_comment() {
        for s in (b'a' ..= b'z').map(|x| char::to_string(&char::from(x))) {
            let mut comment_str = "//".to_string();
            comment_str.push_str(&s);
            assert_eq!(parse_comment(&comment_str), Ok(("", SourceBlock::Comment(s.to_string()))))
        }
    }
}

// var(arg0, arg1, .., argN) with 0 < N
fn parse_app(input: &str) -> IResult<&str, App> {
    pair(parse_var,
         delimited(
            char('('),
            separated_list1(whitespace_delimited(tag(",")), parse_expression),
            char(')'),
         ))(input)
        .map(|(i, o)| (i, App {
            function: o.0,
            args: o.1,
        }))
}

#[cfg(test)]
mod test_parse_app {
    use super::*;

    #[test]
    fn test_zero_argument_app() {
        for s in (b'a' ..= b'z').map(|x| char::to_string(&char::from(x))) {
            let mut app_str = s.to_string();
            app_str.push_str("()");
            assert_eq!(parse_app(&app_str).ok(), None)
        }
    }

    #[test]
    fn test_single_argument_app() {
        for s_function in (b'a' ..= b'z').map(|x| char::to_string(&char::from(x))) {
            for s_arg in (b'a' ..= b'z').map(|x| char::to_string(&char::from(x))) {
                let mut app_str = s_function.clone().to_string();
                app_str.push_str("(");
                app_str.push_str(&s_arg);
                app_str.push_str(")");
                assert_eq!(parse_app(&app_str), Ok(("", App {
                    function: s_function.clone(),
                    args: vec![Expr::Var(s_arg)],
                })))
            }
        }
    }

    #[test]
    fn test_two_argument_app() {
        for s_function in (b'a' ..= b'z').map(|x| char::to_string(&char::from(x))) {
            for s_arg_1 in (b'a' ..= b'z').map(|x| char::to_string(&char::from(x))) {
                for s_arg_2 in (b'a' ..= b'z').map(|x| char::to_string(&char::from(x))) {
                    for spaces_1 in vec!["", " "] {
                        for spaces_2 in vec!["", " "] {
                            let mut app_str = s_function.clone().to_string();
                            app_str.push_str(&"(");
                            app_str.push_str(&s_arg_1);
                            app_str.push_str(&spaces_1);
                            app_str.push_str(&",");
                            app_str.push_str(&spaces_2);
                            app_str.push_str(&s_arg_2);
                            app_str.push_str(&")");

                            assert_eq!(parse_app(&app_str), Ok(("", App {
                                function: s_function.clone(),
                                args: vec![Expr::Var(s_arg_1.clone()), Expr::Var(s_arg_2.clone())],
                            })))
                        }
                    }
                }
            }
        }
    }
}

// TODO
fn parse_elem_literal(input: &str) -> IResult<&str, String> {
    parse_string(input)
        .map(|(i, o)| (i, o.to_string()))
}

#[cfg(test)]
mod test_parse_elem_literal {
    use super::*;

    #[test]
    fn test_string_no_escapes() {
        assert_eq!(parse_elem_literal("\"\""), Ok(("", "".to_string())));
        assert_eq!(parse_elem_literal("\"Unit\""), Ok(("", "Unit".to_string())));
        assert_eq!(parse_elem_literal("\"''\""), Ok(("", "''".to_string())));
        assert_eq!(parse_elem_literal("\"[1, 2, 3]\""), Ok(("", "[1, 2, 3]".to_string())));
    }

    #[test]
    fn test_string_escapes() {
        assert_eq!(parse_elem_literal("\"\r\n\t\""), Ok(("", "\r\n\t".to_string())));
    }

    #[test]
    fn test_string_escapes_failing() {
        assert_eq!(parse_elem_literal("\"\\\""), Ok(("", "\\".to_string())));
    }
}

fn parse_expression(input: &str) -> IResult<&str, Expr> {
    alt((parse_app.map(|o| Expr::App(o)),
         parse_elem_literal.map(|o| Expr::Lit(o)),
         parse_var.map(|o| Expr::Var(o))
    ))(input)
}

fn parse_assignment(input: &str) -> IResult<&str, SourceBlock> {
    pair(parse_var,
        preceded(whitespace_delimited(tag("=")),
                 parse_app))(input)
        .map(|(i, o)| (i, SourceBlock::Assignment(Assignment {
            var: o.0,
            app: o.1
        })))
}

#[cfg(test)]
mod test_parse_assignment {
    use super::*;

    #[test]
    fn test_assignments() {
        assert_eq!(parse_assignment("foo = convolve(bar, baz(two, \"hi\"))"), Ok(("", SourceBlock::Assignment(Assignment {
            var: "foo".to_string(),
            app: App {
                function: "convolve".to_string(),
                args: vec![
                    Expr::Var("bar".to_string()),
                    Expr::App(App {
                        function: "baz".to_string(),
                        args: vec![
                            Expr::Var("two".to_string()),
                            Expr::Lit("hi".to_string())
                        ],
                    }),
                ],
            },
        }))))
    }
}

fn parse_source_block(input: &str) -> IResult<&str, SourceBlock> {
    preceded(multispace0, alt((parse_comment, parse_assignment)))(input)
}

/// Parse a cryptoscript program as a series of assignments of the form:
/// "var = function(arg0, arg1, .., argN)"
pub fn parse_nom(input: &str) -> IResult<&str, SourceCode> {
    many0(parse_source_block)(input)
        .map(|(i, o)| (i, SourceCode { blocks: o }))
}

// utils
// TODO: relocate

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
fn whitespace_delimited<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
  where
  F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(
    multispace0,
    inner,
    multispace0
  )
}

fn parens_delimited<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
  where
  F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(
    char('('),
    inner,
    char(')'),
  )
}


// parse escapable string, from nom examples:

// parser combinators are constructed from the bottom up:
// first we write parsers for the smallest elements (escaped characters),
// then combine them into larger parsers.

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6
/// hexadecimal numerals. We will combine this later with parse_escaped_char
/// to parse sequences like \u{00AC}.
fn parse_unicode<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
  E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
  // `take_while_m_n` parses between `m` and `n` bytes (inclusive) that match
  // a predicate. `parse_hex` here parses between 1 and 6 hexadecimal numerals.
  let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());

  // `preceded` takes a prefix parser, and if it succeeds, returns the result
  // of the body parser. In this case, it parses u{XXXX}.
  let parse_delimited_hex = preceded(
    char('u'),
    // `delimited` is like `preceded`, but it parses both a prefix and a suffix.
    // It returns the result of the middle parser. In this case, it parses
    // {XXXX}, where XXXX is 1 to 6 hex numerals, and returns XXXX
    delimited(char('{'), parse_hex, char('}')),
  );

  // `map_res` takes the result of a parser and applies a function that returns
  // a Result. In this case we take the hex bytes from parse_hex and attempt to
  // convert them to a u32.
  let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));

  // map_opt is like map_res, but it takes an Option instead of a Result. If
  // the function returns None, map_opt returns an error. In this case, because
  // not all u32 values are valid unicode code points, we have to fallibly
  // convert to char with from_u32.
  map_opt(parse_u32, |value| std::char::from_u32(value))(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
  E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
  preceded(
    char('\\'),
    // `alt` tries each parser in sequence, returning the result of
    // the first successful match
    alt((
      parse_unicode,
      // The `value` parser returns a fixed value (the first argument) if its
      // parser (the second argument) succeeds. In these cases, it looks for
      // the marker characters (n, r, t, etc) and returns the matching
      // character (\n, \r, \t, etc).
      value('\n', char('n')),
      value('\r', char('r')),
      value('\t', char('t')),
      value('\u{08}', char('b')),
      value('\u{0C}', char('f')),
      value('\\', char('\\')),
      value('/', char('/')),
      value('"', char('"')),
    )),
  )(input)
}

/// Parse a backslash, followed by any amount of whitespace. This is used later
/// to discard any escaped whitespace.
fn parse_escaped_whitespace<'a, E: ParseError<&'a str>>(
  input: &'a str,
) -> IResult<&'a str, &'a str, E> {
  preceded(char('\\'), multispace1)(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
  // `is_not` parses a string of 0 or more characters that aren't one of the
  // given characters.
  let not_quote_slash = is_not("\"\\");

  // `verify` runs a parser, then runs a verification function on the output of
  // the parser. The verification function accepts out output only if it
  // returns true. In this case, we want to ensure that the output of is_not
  // is non-empty.
  verify(not_quote_slash, |s: &str| !s.is_empty())(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
  Literal(&'a str),
  EscapedChar(char),
  EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a, E>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
where
  E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
  alt((
    // The `map` combinator runs a parser, then applies a function to the output
    // of that parser.
    map(parse_literal, StringFragment::Literal),
    map(parse_escaped_char, StringFragment::EscapedChar),
    value(StringFragment::EscapedWS, parse_escaped_whitespace),
  ))(input)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
fn parse_string<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
where
  E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
  // fold_many0 is the equivalent of iterator::fold. It runs a parser in a loop,
  // and for each output value, calls a folding function on each output value.
  let build_string = fold_many0(
    // Our parser function– parses a single string fragment
    parse_fragment,
    // Our init value, an empty string
    String::new,
    // Our folding function. For each fragment, append the fragment to the
    // string.
    |mut string, fragment| {
      match fragment {
        StringFragment::Literal(s) => string.push_str(s),
        StringFragment::EscapedChar(c) => string.push(c),
        StringFragment::EscapedWS => {}
      }
      string
    },
  );

  // Finally, parse the string. Note that, if `build_string` could accept a raw
  // " character, the closing delimiter " would never match. When using
  // `delimited` with a looping parser (like fold_many0), be sure that the
  // loop won't accidentally match your closing delimiter!
  delimited(char('"'), build_string, char('"'))(input)
}

