use std::collections::HashMap;
use std::io::{BufRead, Write};

use crate::frontend::expression::{Expression, OrderType};
use crate::frontend::instruction::{Instruction, WhileType, Writable};

pub struct ExecutionContext<'a> {
    pub integers: HashMap<&'a str, i32>,
    pub input: &'a mut dyn BufRead,
    pub output: &'a mut dyn Write,
}

pub fn execute_program<'a>(
    program: &'a [Instruction<'a>],
    execution_context: &mut ExecutionContext<'a>,
) -> Result<(), ExecutionError> {
    for instr in program {
        instr.execute(execution_context)?;
    }
    Ok(())
}

fn execute_block<'a>(
    block: &'a [Instruction<'a>],
    execution_context: &mut ExecutionContext<'a>,
) -> Result<(), ExecutionError> {
    for instr in block {
        instr.execute(execution_context)?;
    }
    Ok(())
}

impl<'a> Instruction<'a> {
    fn execute(
        &'a self,
        execution_context: &mut ExecutionContext<'a>,
    ) -> Result<(), ExecutionError> {
        match self {
            Instruction::Read(vars) => {
                for var in vars.iter() {
                    let mut input = String::new();
                    execution_context
                        .input
                        .read_line(&mut input)
                        .map_err(|_| ExecutionError::ReadingError)?;

                    let value: i32 = input
                        .trim()
                        .parse()
                        .map_err(|_| ExecutionError::IntegerParsingError)?;
                    execution_context.integers.insert(var, value);
                }
                Ok(())
            }
            Instruction::Assignment(var, expr) => {
                let val = expr.evaluate(execution_context)?;
                execution_context.integers.insert(var, val);
                Ok(())
            }
            Instruction::Write(writables) => {
                for writable in writables {
                    match writable {
                        Writable::Expression(expr) => {
                            let value = expr.evaluate(execution_context)?;
                            execution_context
                                .output
                                .write_all(value.to_string().as_bytes())
                                .unwrap();
                        }
                        Writable::String(str) => {
                            execution_context.output.write_all(str.as_bytes()).unwrap()
                        }
                    }
                }
                execution_context.output.write_all(b"\n").unwrap();
                Ok(())
            }
            Instruction::If(cond, if_block, else_block) => {
                let block = if cond.evaluate(execution_context)? != 0 {
                    Some(if_block)
                } else {
                    else_block.as_ref()
                };
                if let Some(block) = block {
                    for instr in block {
                        instr.execute(execution_context)?;
                    }
                }
                Ok(())
            }
            Instruction::While(while_type, cond, block) => {
                match while_type {
                    WhileType::While => {
                        while cond.evaluate(execution_context)? != 0 {
                            execute_block(block, execution_context)?;
                        }
                    }
                    WhileType::DoWhile => {
                        execute_block(block, execution_context)?;
                        while cond.evaluate(execution_context)? != 0 {
                            execute_block(block, execution_context)?;
                        }
                    }
                    WhileType::Repeat => {
                        execute_block(block, execution_context)?;
                        while cond.evaluate(execution_context)? == 0 {
                            execute_block(block, execution_context)?;
                        }
                    }
                };
                Ok(())
            }
            Instruction::For {
                variable,
                start_expr,
                end_expr,
                step,
                block,
            } => {
                let initial = start_expr.evaluate(execution_context)?;
                execution_context.integers.insert(variable, initial);
                let step_value = step.evaluate(execution_context)?;
                if step_value >= 0 {
                    while *execution_context.integers.get(variable).unwrap()
                        <= end_expr.evaluate(execution_context)?
                    {
                        for instr in block {
                            instr.execute(execution_context)?;
                        }
                        execution_context
                            .integers
                            .entry(variable)
                            .and_modify(|e| *e += step_value);
                        if step.evaluate(execution_context)? != step_value {
                            return Err(ExecutionError::VariableStepInLoop);
                        }
                    }
                } else {
                    while *execution_context.integers.get(variable).unwrap()
                        >= end_expr.evaluate(execution_context)?
                    {
                        for instr in block {
                            instr.execute(execution_context)?;
                        }
                        execution_context
                            .integers
                            .entry(variable)
                            .and_modify(|e| *e += step_value);
                    }
                }
                Ok(())
            }
        }
    }
}

impl<'a> Expression<'a> {
    fn evaluate(
        &self,
        execution_context: &mut ExecutionContext<'a>,
    ) -> Result<i32, ExecutionError> {
        match self {
            &Expression::Constant(x) => Ok(x as i32),
            &Expression::Variable(var) => {
                let x = *execution_context
                    .integers
                    .get(var)
                    .ok_or_else(|| ExecutionError::VariableNotDefinedError(var.to_string()))?;
                Ok(x)
            }
            Expression::Minus(expr) => {
                let val = expr.evaluate(execution_context)?;
                Ok(-val)
            }
            Expression::Addition(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context)?;
                let val2 = expr2.evaluate(execution_context)?;
                Ok(val1 + val2)
            }
            Expression::Subtraction(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context)?;
                let val2 = expr2.evaluate(execution_context)?;
                Ok(val1 - val2)
            }
            Expression::Multiplication(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context)?;
                let val2 = expr2.evaluate(execution_context)?;
                Ok(val1 * val2)
            }
            Expression::Division(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context)?;
                let val2 = expr2.evaluate(execution_context)?;
                if val2 == 0 {
                    Err(ExecutionError::ZeroDivisionError)
                } else {
                    Ok(val1 / val2)
                }
            }
            Expression::Reminder(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context)?;
                let val2 = expr2.evaluate(execution_context)?;
                if val2 == 0 {
                    Err(ExecutionError::ZeroDivisionError)
                } else {
                    Ok(val1 % val2)
                }
            }
            Expression::Order(order_type, expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context)?;
                let val2 = expr2.evaluate(execution_context)?;
                let cond = match order_type {
                    OrderType::Less => val1 < val2,
                    OrderType::LessOrEqual => val1 <= val2,
                    OrderType::Equal => val1 == val2,
                    OrderType::GreaterOrEqual => val1 >= val2,
                    OrderType::Greater => val1 > val2,
                };
                if cond {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    ZeroDivisionError,
    VariableNotDefinedError(String),
    ReadingError,
    IntegerParsingError,
    VariableStepInLoop,
}
