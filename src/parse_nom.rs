// use crate::elem::Elem;
use crate::parse_utils::{whitespace_delimited, parse_string};
use crate::restack::{Restack, StackIx};
use crate::untyped_instruction::Instruction;
use crate::untyped_instructions::Instructions;

// use std::sync::Arc;
use std::collections::BTreeSet;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0};
use nom::combinator::recognize;
use nom::multi::{many0, many0_count, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated};
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

/// unpack_json type annotation
pub type TypeAnnotation = String;

/// A parsed source file
#[derive(Debug, Clone, PartialEq)]
pub struct SourceCode {
    /// Vec of SourceBlock's, in order
    blocks: Vec<SourceBlock>,
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

    /// Optional (unpack_json) type annotation
    type_annotation: Option<TypeAnnotation>,

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

// TODO: support '!' ending variables?
//
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
            type_annotation: None,
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
                    type_annotation: None,
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
                                type_annotation: None,
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

    // TODO: support \\ in strings
    // #[test]
    // fn test_string_escapes_failing() {
    //     assert_eq!(parse_elem_literal("\"\\\""), Ok(("", "\\".to_string())));
    // }
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
                type_annotation: None,
                args: vec![
                    Expr::Var("bar".to_string()),
                    Expr::App(App {
                        function: "baz".to_string(),
                        type_annotation: None,
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
    terminated(many0(parse_source_block), multispace0)(input)
        .map(|(i, o)| (i, SourceCode { blocks: o }))
}

#[cfg(test)]
mod test_parse_source_code {
    use super::*;

    #[test]
    fn test_source_code_demo() {
        let test_code_str = r#"
            input_json = unpack_json(INPUT)

            queries = unpack_json(lookup("queries", input_json))
            first_query = unpack_json(index("0", queries))

            _ = assert(check_eq(unpack_json(lookup("action", first_query)), "tokenbalance"))
            _ = assert(check_eq(unpack_json(lookup("contractaddress", first_query)), "0x57d90b64a1a57749b0f932f1a3395792e12e7055"))
            _ = assert(check_eq(unpack_json(lookup("result"), unpack_json(lookup("response", first_query))), "135499"))

            prompts = unpack_json(lookup("prompts", input_json))
            first_prompt = unpack_json(lookup("0", prompts))

            _ = assert(check_eq(unpack_json(lookup("action", first_prompt)), "siwe"))
            _ = assert(check_eq(unpack_json(lookup("version", first_prompt)), "1.1.0"))
            _ = assert(check_eq(unpack_json(lookup("address", unpack_json(lookup("fields", unpack_json(lookup("data", first_prompt)))))), "0xe04f27eb70e025b78871a2ad7eabe85e61212761"))

            message_hash = hash_sha256(string_to_bytes(unpack_json(lookup("message", unpack_json(lookup("data", first_prompt))))))
            address_hash = hash_sha256(string_to_bytes(unpack_json(lookup("address", unpack_json(lookup("fields", unpack_json(lookup("data", first_prompt))))))))

            // # Hex vs list of bytes? Infix?
            // # assert!(hash_sha256(concat(message_hash, address_hash)) == [53,163,178,139,122,187,171,47,42,135,175,176,240,11,10,152,228,238,106,205,132,68,80,79,188,54,124,242,97,132,31,139])
            // # assert!(hash_sha256(concat(message_hash, address_hash)) == 0x35a3b28b7abbab2f2a87afb0f00b0a98e4ee6acd8444504fbc367cf261841f8b)
            _ = assert(check_eq(hash_sha256(concat(message_hash, address_hash)), "0x35a3b28b7abbab2f2a87afb0f00b0a98e4ee6acd8444504fbc367cf261841f8b"))
        "#;

        assert_eq!(parse_nom(&test_code_str).map(|(i, _o)| (i, ())), Ok(("", ())))
    }
}


///////////////////////////
// NOM AST TO STACK AST
//
// TODO: relocate
///////////////////////////

struct InstructionsWriter {
    // defined_vars: BTreeMap<Var, ()>,
    context: Vec<Var>,
    instructions: Instructions,
    // stack_size: usize,
}

impl InstructionsWriter {
    // writer.push("INPUT");

    pub fn new() -> Self {
        Self {
            context: vec![],
            instructions: Instructions::new(),

            // BTreeMap::new(),
            // stack_size: 0,
        }
    }

    /// Get the StackIx of the Var, or throw an error
    pub fn get_var(&self, var: Var) -> Result<StackIx, SourceCodeError> {
        self.context.iter()
            .enumerate()
            .find(|(_i, &var_i)| var_i == var)
            .map(|(i, _var_i)| i)
            .ok_or_else(|| SourceCodeError::InstructionsWriterGetVar {
                context: self.context,
                var: var,
            })
    }

    /// New "temp_N" var with "N = max temp var index + 1"
    pub fn new_var(&self) -> Var {
        let max_var = self.context
            .iter()
            .filter_map(|var| {
                let mut var_temp = var.clone();
                let var_index = var_temp.split_off(5);
                if var_temp == "temp_" {
                    var_index.parse::<u64>().ok()
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);
        format!("temp_{:?}", max_var + 1)
    }

    /// Restack so that the rhs is now in the lhs's slot as well
    ///
    /// lhs = rhs
    pub fn assign(&mut self, lhs: Var, rhs: Var) -> Result<(), SourceCodeError> {
        let rhs_ix = self.get_var(rhs)?;
        let restack_vec = self.context.iter().enumerate().map(|(i, &var_i)| if var_i == lhs {
            rhs_ix
        } else {
            i
        }).collect::<Vec<StackIx>>();
        self.instructions.instructions.push(Instruction::Restack(Restack {
            restack_depth: restack_vec.len(),
            restack_vec: restack_vec,
        }));

        // TODO: update context
        // Ok(())
    }

    /// Restack so that the needed_vars are at the top of the stack (not dropping any)
    pub fn restack(&mut self, needed_vars: Vec<Var>) -> Result<(), SourceCodeError> {
        let mut restacked_vars = BTreeSet::new();

        let restack_vec = needed_vars.iter()
            .map(|needed_var| {
                restacked_vars.insert(needed_var);
                self.get_var(needed_var.to_string())
            })
            .chain(self.context
                   .iter()
                   .enumerate()
                   .filter_map(|(i, var_i)| if restacked_vars.contains(var_i) { None } else { Some(Ok(i)) }))
            .collect::<Result<Vec<StackIx>, SourceCodeError>>()?;

        self.instructions.instructions.push(Instruction::Restack(Restack {
            restack_depth: restack_vec.len(),
            restack_vec: restack_vec,
        }));

        // TODO: update context
        // Ok(())
    }

    /// Push the instruction with the Var's name and optional TypeAnnotation
    ///
    /// Return the output variable
    pub fn instruction(&mut self, function: Var, type_annotation: Option<TypeAnnotation>) -> Result<Var, SourceCodeError> {

    }

    pub fn finalize(&self) -> Result<Instructions, SourceCodeError> {
        Ok(self.instructions)
    }
}

impl Expr {
    /// Output variable representing the arg
    pub fn to_instructions(&self, writer: &mut InstructionsWriter) -> Result<Var, SourceCodeError> {

    }
}

impl App {
    /// Output variable returned
    pub fn to_instructions(&self, writer: &mut InstructionsWriter) -> Result<Var, SourceCodeError> {
        let needed_vars = self.args
            .iter()
            .map(|arg| arg.to_instructions(writer))
            .collect::<Result<Vec<Var>, SourceCodeError>>()?;
        writer.restack(needed_vars);
        writer.instruction(self.function, self.type_annotation)

        // 1. iterate through args
        //     lit => push => var
        //     app => new temp var => var
        //     var => var
        // 2. restack so that list of vars == list on stack
        // 3. output that instruction
    }
}


impl Assignment {
    pub fn to_instructions(&self, writer: &mut InstructionsWriter) -> Result<(), SourceCodeError> {
        let output_var = self.app.to_instructions(writer)?;
        writer.assign(self.var, output_var)
    }
}

impl SourceBlock {
    pub fn to_instructions(&self, writer: &mut InstructionsWriter) -> Result<(), SourceCodeError> {
        match self {
            Self::Comment(_) => Ok(()),
            Self::Assignment(assignment) => assignment.to_instructions(writer),
        }
    }
}

impl SourceCode {
    pub fn to_instructions(&self) -> Result<Instructions, SourceCodeError> {
        let mut writer = InstructionsWriter::new();
        for block in self.blocks {
            block.to_instructions(&mut writer)?
        }
        writer.finalize()
    }
}

pub enum SourceCodeError {
    InstructionsWriterGetVar {
        context: Vec<Var>,
        var: Var,
    },
}


