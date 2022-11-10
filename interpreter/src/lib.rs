mod backend;
mod frontend;

use backend::{execute_program, ExecutionError};
use frontend::instruction;
use std::{collections::HashMap, io::{Write, BufRead}};

use backend::ExecutionContext;
use nom::error::VerboseError;

pub fn interpret<'a>(
    input: &'a mut dyn BufRead,
    output: &'a mut dyn Write, 
    program_string: &'a str
) -> Result<(), InterpreterError<'a>> {
    let  (_, program) = instruction::program::<VerboseError<&str>>(program_string)?;
    Box::leak(Box::new(32));
    let mut execution_context = ExecutionContext {
        integers: HashMap::new(),
        input,
        output
    };
    execute_program(&program, &mut execution_context)?;
    Ok(())
}

#[derive(Debug)]
pub enum InterpreterError<'a> {
    ParsingError(VerboseError<&'a str>),
    ExecutionError(ExecutionError)
}

impl<'a> From<VerboseError<&'a str>> for InterpreterError<'a> {
    fn from(e: VerboseError<&'a str>) -> Self {
        InterpreterError::ParsingError(e)
    }
}

impl<'a> From<ExecutionError> for InterpreterError<'a> {
    fn from(e: ExecutionError) -> Self {
        InterpreterError::ExecutionError(e)
    }
}