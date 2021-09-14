mod backend;
mod frontend;

use backend::execute_program;
use frontend::instruction;
use std::{collections::HashMap, env, fs, path::Path};

use crate::backend::ExecutionContext;
use nom::{
    Err,
    error::{VerboseError, convert_error},
};

fn main() {
    // Read contents of the file
    let file_name = env::args().nth(1).expect("Provide a file name");
    let path = Path::new(&file_name);
    let mut i = fs::read_to_string(path).expect("Invalid file");
    i.insert(0, '\n');
    let i: &str = &i;
    dbg!(i);

    // Parse the contents of the file
    let result = instruction::program::<VerboseError<&str>>(i);

    let (_, program) = result.unwrap_or_else(|e| {
        if let Err::Error(e) | Err::Failure(e) = e {
            let error_message = convert_error(i, e);
            panic!("{}", error_message);
        };
        panic!("asd");
    }
    );
    dbg!(&program);

    // Execute the program
    let mut execution_context = ExecutionContext {
        integers: HashMap::new(),
    };
    println!("Executing...");
    execute_program(&program, &mut execution_context);
    println!("\nDone!");
}
