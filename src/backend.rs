use std::collections::HashMap;
use std::convert::TryInto;

use crate::frontend::expression::Expression;
use crate::frontend::instruction::{Block, Instruction};

pub struct ExecutionContext<'a> {
    pub integers: HashMap<&'a str, i32>,
}

pub fn execute_program<'a>(program: Block<'a>, execution_context: &mut ExecutionContext<'a>) {
    program
        .iter()
        .for_each(|instr| instr.execute(execution_context))
}

impl<'a> Instruction<'a> {
    fn execute(&self, execution_context: &mut ExecutionContext<'a>) {
        match self {
            Instruction::Read(vars) => {
                let mut input = String::new();
                for var in vars.iter() {
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Could not read input!");
                    let value: i32 = input.trim().parse().expect("Expected an integer!");
                    execution_context.integers.insert(var, value);
                }
            }
            Instruction::Assignment(var, expr) => {
                let val = expr.evaluate(execution_context);
                execution_context.integers.insert(var, val);
            }
            Instruction::Write(expr) => {
                println!("{}", expr.evaluate(execution_context));
            }
            Instruction::If(cond, if_block, else_block) => {
                let block = if cond.evaluate(execution_context) != 0 {
                    Some(if_block)
                } else {
                    else_block.as_ref()
                };
                if let Some(block) = block {
                    block
                        .iter()
                        .for_each(|instr| instr.execute(execution_context));
                }
            }
            Instruction::While(cond, block) => {
                while cond.evaluate(execution_context) != 0 {
                    block
                        .iter()
                        .for_each(|instr| instr.execute(execution_context));
                }
            }
        }
    }
}

impl<'a> Expression<'a> {
    fn evaluate(&self, execution_context: &mut ExecutionContext<'a>) -> i32 {
        match self {
            &Expression::Constant(x) => (x as u32).try_into().unwrap(),
            &Expression::Variable(var) => {
                let x = *execution_context
                    .integers
                    .get(var)
                    .unwrap_or_else(|| panic!("Variable {} not defined!", var));
                x
            }
            Expression::Minus(expr) => {
                let val = expr.evaluate(execution_context);
                -val
            }
            Expression::Addition(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context);
                let val2 = expr2.evaluate(execution_context);
                val1 + val2
            }
            Expression::Subtraction(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context);
                let val2 = expr2.evaluate(execution_context);
                val1 - val2
            }
            Expression::Multiplication(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context);
                let val2 = expr2.evaluate(execution_context);
                val1 * val2
            }
            Expression::Division(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context);
                let val2 = expr2.evaluate(execution_context);
                if val2 == 0 {
                    panic!("Cannot divide by 0");
                }
                val1 / val2
            }
            Expression::Reminder(expr1, expr2) => {
                let val1 = expr1.evaluate(execution_context);
                let val2 = expr2.evaluate(execution_context);
                if val2 == 0 {
                    panic!("Cannot divide by 0");
                }
                val1 % val2
            }
        }
    }
}
