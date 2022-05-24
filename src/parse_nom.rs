use crate::elem::{Elem, ElemSymbol};
use crate::elems::ElemsPopError;
use crate::parse_utils::{whitespace_delimited, parse_string};
use crate::restack::{Restack, StackIx};
use crate::untyped_instruction::{Instruction, InstructionError};
use crate::untyped_instructions::Instructions;
use crate::typed_instr::Instr;

use std::cmp;
// use std::sync::Arc;
// use std::collections::BTreeSet;

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
pub struct SourceCode<T> {
    /// Vec of SourceBlock's, in order
    blocks: Vec<SourceBlock<T>>,
}

/// A single block of parsed source code
#[derive(Debug, Clone, PartialEq)]
pub enum SourceBlock<T> {
    /// A line comment
    Comment(Comment),

    /// An assignment, which could span multiple lines
    Assignment(Assignment<T>),
}

/// A single assignment: assignments are "simple," i.e. no pattern matching, etc
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment<T> {
    /// Assigned variable
    var: Var,

    /// Expression assigned
    app: App<T>,
}

/// An application of a function to a Vec of arguments
#[derive(Debug, Clone, PartialEq)]
pub struct App<T> {
    /// The function variable: "f" in "f(1, 2, 3)"
    function: Var,

    /// Optional (unpack_json) type annotation
    type_annotation: Option<TypeAnnotation>,

    /// The argument expressions: "[1, 2, 3]" in "f(1, 2, 3)"
    args: Vec<Expr<T>>,
}

/// Parsed expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<T> {
    /// Function application
    App(App<T>),

    /// Literal
    // Lit(String),
    Lit(T),

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

fn parse_comment<T>(input: &str) -> IResult<&str, SourceBlock<T>> {
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
            assert_eq!(parse_comment::<String>(&comment_str), Ok(("", SourceBlock::Comment(s.to_string()))))
        }
    }
}

// var(arg0, arg1, .., argN) with 0 < N
fn parse_app(input: &str) -> IResult<&str, App<String>> {
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

fn parse_expression(input: &str) -> IResult<&str, Expr<String>> {
    alt((parse_app.map(|o| Expr::App(o)),
         parse_elem_literal.map(|o| Expr::Lit(o)),
         parse_var.map(|o| Expr::Var(o))
    ))(input)
}

fn parse_assignment(input: &str) -> IResult<&str, SourceBlock<String>> {
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

fn parse_source_block(input: &str) -> IResult<&str, SourceBlock<String>> {
    preceded(multispace0, alt((parse_comment, parse_assignment)))(input)
}

/// Parse a cryptoscript program as a series of assignments of the form:
/// "var = function(arg0, arg1, .., argN)"
pub fn parse_nom(input: &str) -> IResult<&str, SourceCode<String>> {
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

// API:
// - 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionsWriter {
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
    pub fn get_var(&self, var: &Var) -> Result<StackIx, SourceCodeError> {
        self.context.iter()
            .enumerate()
            .find(|(_i, var_i)| *var_i == var)
            .map(|(i, _var_i)| i)
            .ok_or_else(|| SourceCodeError::InstructionsWriterGetVar {
                context: self.context.clone(),
                var: var.clone(),
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
    pub fn assign(&mut self, lhs: Var, rhs: &Var) -> Result<(), SourceCodeError> {
        let rhs_ix = self.get_var(rhs)?;
        let assigned_context = self.context.iter().enumerate().map(|(i, var_i)| if *var_i == lhs {
            (rhs_ix, rhs)
        } else {
            (i, var_i)
        });

        let restack_vec = assigned_context.clone()
            .map(|(i, _var_i)| i)
            .collect::<Vec<StackIx>>();
        self.context = assigned_context.clone()
            .map(|(_i, var_i)| var_i.clone())
            .collect::<Vec<Var>>();

        self.instructions.instructions.push(Instruction::Restack(Restack {
            restack_depth: restack_vec.len(),
            restack_vec: restack_vec,
        }));

        Ok(())
    }

    /// Restack so that the needed_vars are at the top of the stack (without dropping any or
    /// modifying the context)
    pub fn restack_for_instruction(&mut self, needed_vars: Vec<Var>) -> Result<(), SourceCodeError> {
        let largest_restacked_var: Option<usize> = needed_vars.iter()
            .try_fold(None, |current_largest_restacked_var, needed_var| {
                let var_index = self.get_var(needed_var)?;
                Ok::<Option<usize>, SourceCodeError>(Some(cmp::max(var_index, current_largest_restacked_var.unwrap_or(0))))
            })?;
        let restack_vec = needed_vars.iter()
            .map(|needed_var| self.get_var(needed_var))
            .chain(largest_restacked_var
                   .map(|restacked_vars| (0..=restacked_vars))
                   .unwrap_or_else(|| (1..=0))
                   .into_iter()
                   .map(|i| Ok(i)))
            .collect::<Result<Vec<StackIx>, SourceCodeError>>()?;
        self.instructions.instructions.push(Instruction::Restack(Restack {
            restack_depth: restack_vec.len(),
            restack_vec: restack_vec,
        }));
        Ok(())
    }

    pub fn var_to_instruction(function: Var, opt_type_annotation: Option<TypeAnnotation>) -> Result<Instruction, SourceCodeError> {
        match (&*function, opt_type_annotation.clone()) {
            ("hash_sha256", None) => Ok(Instruction::HashSha256),
            ("check_le", None) => Ok(Instruction::CheckLe),
            ("check_lt", None) => Ok(Instruction::CheckLt),
            ("check_eq", None) => Ok(Instruction::CheckEq),
            ("string_eq", None) => Ok(Instruction::StringEq),
            ("bytes_eq", None) => Ok(Instruction::BytesEq),
            ("concat", None) => Ok(Instruction::Concat),
            ("slice", None) => Ok(Instruction::Slice),
            ("index", None) => Ok(Instruction::Index),
            ("lookup", None) => Ok(Instruction::Lookup),
            ("assert_true", None) => Ok(Instruction::AssertTrue),
            ("to_json", None) => Ok(Instruction::ToJson),
            ("unpack_json", Some(type_annotation)) => {

                let elem_symbol = match &*type_annotation {
                    "Unit" => Ok(ElemSymbol::Unit),
                    "Bool" => Ok(ElemSymbol::Bool),
                    "Number" => Ok(ElemSymbol::Number),
                    "Bytes" => Ok(ElemSymbol::Bytes),
                    "String" => Ok(ElemSymbol::String),
                    "Array" => Ok(ElemSymbol::Array),
                    "Object" => Ok(ElemSymbol::Object),
                    "Json" => Ok(ElemSymbol::Json),
                    _ => Err(SourceCodeError::VarToInstructionUnknownType(type_annotation)),
                }?;
                Ok(Instruction::UnpackJson(elem_symbol))
            },
            ("string_to_bytes", None) => Ok(Instruction::StringToBytes),
            (_, Some(type_annotation)) =>
                Err(SourceCodeError::VarToInstructionExtraAnnotation {
                    function: function,
                    type_annotation: type_annotation
                }),
            _ => Err(SourceCodeError::VarToInstructionUnknownFunction {
                function: function,
                opt_type_annotation: opt_type_annotation,
            }),
        }
    }

    /// - Push the instruction with the Var's name and optional TypeAnnotation
    /// - Consume the variables from the stack
    /// - Return the output variable (after pushing onto the stack)
    pub fn instruction(&mut self, function: Var, type_annotation: Option<TypeAnnotation>) -> Result<Var, SourceCodeError> {
        let instruction = Self::var_to_instruction(function, type_annotation)?;
        let instr = instruction.clone().to_instr()?;
        let instr_type = match instr {
            Instr::Instr(instr2) => Ok(instr2.type_of()?),
            Instr::Restack(restack) => Err(SourceCodeError::InstructionRestackUnexpected(restack)),
        }?;

        // instruction written to log
        self.instructions.push(instruction);
        // variables consumed by instruction
        self.context.drain(0..instr_type.i_type.len());
        let output_var = self.new_var();
        // variable produced by instruction
        self.context.insert(0, output_var.clone());
        Ok(output_var)
    }

    pub fn finalize(&self) -> Result<Instructions, SourceCodeError> {
        Ok(self.instructions.clone())
    }
}

impl Expr<Elem> {
    /// Output variable representing the arg
    pub fn to_instructions(&self, writer: &mut InstructionsWriter) -> Result<Var, SourceCodeError> {
        match self {
            Self::App(app) => app.to_instructions(writer),
            Self::Lit(lit) => {
                let new_var = writer.new_var();
                writer.instructions.push(Instruction::Push(lit.clone()));
                writer.context.insert(0, new_var.clone());
                Ok(new_var)
            },
            Self::Var(var) => {
                let var_string = var.to_string();
                writer.get_var(&var_string)?;
                Ok(var_string)
            },
        }

// /// Parsed expression
// #[derive(Debug, Clone, PartialEq)]
// pub enum Expr<T> {
//     /// Function application
//     App(App<T>),

//     /// Literal
//     // Lit(String),
//     Lit(T),

//     /// Variable
//     Var(Var),
// }


    }
}

impl App<Elem> {
    /// Output variable returned
    pub fn to_instructions(&self, writer: &mut InstructionsWriter) -> Result<Var, SourceCodeError> {
        let needed_vars = self.args
            .iter()
            .map(|arg| arg.to_instructions(writer))
            .collect::<Result<Vec<Var>, SourceCodeError>>()?;
        writer.restack_for_instruction(needed_vars)?;
        writer.instruction(self.function.clone(), self.type_annotation.clone())

        // 1. iterate through args
        //     lit => push => var
        //     app => new temp var => var
        //     var => var
        // 2. restack so that list of vars == list on stack
        // 3. output that instruction
    }
}


impl Assignment<Elem> {
    pub fn to_instructions(&self, writer: &mut InstructionsWriter) -> Result<(), SourceCodeError> {
        let output_var = self.app.to_instructions(writer)?;
        writer.assign(self.var.clone(), &output_var)
    }
}

impl SourceBlock<Elem> {
    pub fn to_instructions(&self, writer: &mut InstructionsWriter) -> Result<(), SourceCodeError> {
        match self {
            Self::Comment(_) => Ok(()),
            Self::Assignment(assignment) => assignment.to_instructions(writer),
        }
    }
}

impl SourceCode<Elem> {
    pub fn to_instructions(&self) -> Result<Instructions, SourceCodeError> {
        let mut writer = InstructionsWriter::new();
        for block in &self.blocks {
            block.to_instructions(&mut writer)?
        }
        writer.finalize()
    }
}

#[derive(Debug, Clone)]
pub enum SourceCodeError {
    // var not found in context
    InstructionsWriterGetVar {
        context: Vec<Var>,
        var: Var,
    },

    VarToInstructionUnknownType(TypeAnnotation),

    VarToInstructionExtraAnnotation {
        function: Var,
        type_annotation: TypeAnnotation,
    },

    VarToInstructionUnknownFunction {
        function: Var,
        opt_type_annotation: Option<TypeAnnotation>,
    },

    // TODO: make this error impossible by using a different enum
    // Restack not expected to be able to be generated at this stage
    InstructionRestackUnexpected(Restack),

    /// "ElemsPop failed: \n{0:?}\n"
    // #[error("ElemsPop failed: \n{0:?}\n")]
    ElemsPopError(ElemsPopError),

    /// "Instruction failed: \n{0:?}\n"
    // #[error("Instruction failed: \n{0:?}\n")]
    InstructionError(InstructionError),
}

impl From<ElemsPopError> for SourceCodeError {
    fn from(error: ElemsPopError) -> Self {
        Self::ElemsPopError(error)
    }
}

impl From<InstructionError> for SourceCodeError {
    fn from(error: InstructionError) -> Self {
        Self::InstructionError(error)
    }
}

