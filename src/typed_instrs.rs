use crate::elem::ElemSymbol;
use crate::elem_type::{ElemType, StackType};
use crate::stack::Stack;
use crate::restack::Restack;
use crate::elems::ElemsPopError;
use crate::typed_instruction::{IsStackInstruction, StackInstructionError};
use crate::typed_instr::Instr;

use std::fmt::Debug;
use std::sync::Arc;

use enumset::EnumSet;

/// A list of Instr's. See Instr for more info
#[derive(Clone, Debug)]
pub struct Instrs {
    /// A list of Instr's
    pub instrs: Vec<Instr>,
}

impl Default for Instrs {
    fn default() -> Self {
        Self::new()
    }
}

impl Instrs {
    /// A new empty list of Instr's
    pub fn new() -> Self {
        Instrs {
            instrs: vec![],
        }
    }

    /// Print the list of Instr's for debugging
    pub fn debug(&self) -> Result<(), ElemsPopError> {
        println!("instructions:");
        for (line_no, instruction) in self.instrs.iter().enumerate() {
            println!("#{:?}:", line_no);
            match instruction {
                Instr::Instr(instr) => {
                    println!("{:?}", instr);
                    println!("{}\n", instr.type_of()?);
                },
                Instr::Restack(restack) => {
                    println!("{:?}", restack);
                    println!("{}\n",
                             restack
                             .type_of(From::from(line_no))
                             .map_err(ElemsPopError::RestackError)?);
                },
            }
        }
        println!("--------------------------------------------------------------------------------");
        println!("");
        Ok(())
    }

    /// Assuming an input stack of [Json, Json, ..] (num_input_json count),
    /// what's the monomorphic type of Self?
    pub fn type_of_mono(&self, num_input_json: usize) -> Result<StackType, StackInstructionError> {
        let mut stack_type = (0..num_input_json).map(|_| ElemType::from_locations(EnumSet::only(ElemSymbol::Json), vec![])).collect();
        for (line_no, instr_or_restack) in (&self.instrs).iter().enumerate() {
            println!("------------------------------------------------------------------------------------------");
            println!("line_no: {}", line_no);
            println!("{:?}\n", instr_or_restack);
            match instr_or_restack {
                Instr::Instr(instr) => {
                    let mut instr_type = instr.type_of()
                        .map_err(StackInstructionError::ElemsPopError)?;
                    println!("instr: {}\n", instr_type);
                    stack_type = instr_type.specialize_to_input_stack(stack_type)
                        .map_err(StackInstructionError::TypeError)?;
                },
                Instr::Restack(restack) => {
                    restack.run(&mut stack_type.types)
                        .map_err(StackInstructionError::RestackError)?
                },
            }
        }
        println!("------------------------------------------------------------------------------------------");
        println!("Finished running successfully.\n");
        println!("Final stack:");
        Ok(stack_type)
    }

    /// Run the list of individually-typed instructions. It can fail if adjacent
    /// instructions have non-matching types, e.g. if "Push(true)" is
    /// immediately followed by "UnpackJson".
    pub fn run(&self, stack: &mut Stack) -> Result<(), StackInstructionError> {
        for (line_no, instr_or_restack) in (&self.instrs).iter().enumerate() {
            stack.debug().map_err(|e| StackInstructionError::DebugJsonError(Arc::new(e)))?;
            println!("------------------------------------------------------------------------------------------");
            println!("line_no: {}", line_no);
            println!("{:?}\n", instr_or_restack);
            match instr_or_restack {
                Instr::Instr(instr) => {
                    println!("");
                    stack.debug_type();
                    match instr.type_of() {
                        Ok(instr_type) => {
                            println!("instr: {}\n", instr_type);
                            let mut mut_instr_type = instr_type.clone();
                            match mut_instr_type
                                .specialize_to_input_stack(stack.type_of()) {
                                Ok(_) => println!("specialized: {}\n", mut_instr_type),
                                Err(e) => println!("specialization failed:\n{}\n", e),
                            }
                        },
                        Err(e) => println!("instr type_of errror: {}\n", e),
                    }
                    println!("");
                    instr.stack_run(stack)?
                },
                Instr::Restack(restack) => {
                    restack.run(&mut stack.stack)
                        .map_err(StackInstructionError::RestackError)?
                },
            }
        }
        println!("------------------------------------------------------------------------------------------");
        println!("Finished running successfully.\n");
        println!("Final stack:");
        stack.debug().map_err(|e| StackInstructionError::DebugJsonError(Arc::new(e)))?;
        Ok(())
    }

    /// Push an instruction that IsStackInstruction onto the list of instructions
    pub fn instr(&mut self, instr: impl IsStackInstruction + 'static) -> () {
        self.instrs.push(Instr::Instr(Arc::new(instr)))
    }

    /// Push a Restack onto the list of instructions
    pub fn restack(&mut self, restack: Restack) -> () {
        self.instrs.push(Instr::Restack(restack))
    }
}

