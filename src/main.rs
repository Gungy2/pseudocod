mod backend;
mod frontend;

use backend::execute_program;
use frontend::instruction;
use std::{collections::HashMap, env, fs, path::Path};

use crate::backend::ExecutionContext;

fn main() {
    // Read contents of the file
    let file_name = env::args().nth(1).expect("Provide a file name");
    let path = Path::new(&file_name);
    let mut i = fs::read_to_string(path).expect("Invalid file");
    i.insert(0, '\n');
    dbg!(&i);

    // Parse the contents of the file
    let (rest, program) = instruction::program(&i).expect("Could not parse the program");
    dbg!(&program);
    assert_eq!(rest, "");

    // Execute the program
    let mut execution_context = ExecutionContext {
        integers: HashMap::new(),
    };
    println!("Executing...");
    execute_program(&program, &mut execution_context);
    println!("\nDone!");
}
