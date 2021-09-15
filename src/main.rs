mod backend;
mod frontend;

use backend::execute_program;
use frontend::instruction;
use std::{collections::HashMap, env, fs, path::Path};

use crate::backend::ExecutionContext;
use nom::{
    error::{convert_error, VerboseError},
    Err,
};

fn main() {
    // Read contents of the file
    let file_name = env::args().nth(1).expect("Introduceti numele fisierului!");
    let path = Path::new(&file_name);
    let mut i = fs::read_to_string(path).expect("Fisier invalid!");
    i.insert(0, '\n');
    let i: &str = &i;

    // Parse the contents of the file
    let result = instruction::program::<VerboseError<&str>>(i);

    let (_, program) = result.unwrap_or_else(|e| {
        if let Err::Error(e) | Err::Failure(e) = e {
            let error_message = convert_error(i, e);
            panic!("{}", error_message);
        };
        panic!("Programul nu este valid!");
    });
    dbg!(&program);

    // Execute the program
    let mut execution_context = ExecutionContext {
        integers: HashMap::new(),
    };
    println!("Se executa...");
    execute_program(&program, &mut execution_context);
    println!("\nGata!");
}
