use pseudocod::interpret;
use std::{env, fs, path::Path};

fn main() {
    let file_name = env::args().nth(1).expect("Introduceti numele fisierului");
    let path = Path::new(&file_name);
    let input = fs::read_to_string(path).expect("Fisier invalid");

    interpret(&mut std::io::stdin().lock(), &mut std::io::stdout(), &input).unwrap();
}
