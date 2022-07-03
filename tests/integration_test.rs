use std::{
    fs::read_to_string,
    io::Cursor,
};

use pseudocod::interpret;
use test_case::test_case;

#[test_case("writes.pseudo", "4\n5\n13\n", ""; "write")]
#[test_case("reads.pseudo", "12\n35\n", "1\n2\n3\n4\n5\n"; "read")]
#[test_case("if1.pseudo", "10\n", ""; "simple if")]
#[test_case("if2.pseudo", "5\n6\n", ""; "nested if")]
#[test_case("if3.pseudo", "6\n", ""; "else if")]
#[test_case("do_while.pseudo", "0\n", ""; "do while")]
#[test_case("repeat.pseudo", "0\n", ""; "repeat")]
#[test_case("while.pseudo", "5\n4\n3\n2\n", ""; "while instruction")]
#[test_case("for.pseudo", "0\n2\n4\n6\n8\n10\n12\n", ""; "for instruction")]
#[test_case("fibonacci.pseudo", "Introduceti n:\nfib(10) = 55\n", "10"; "fibonacci")]
fn integration_test(file_name: &str, output: &str, input: &'static str) {
    let path = std::path::Path::new("tests")
        .join("resources")
        .join(file_name);

    let program_string = read_to_string(path).expect("Could not read file");
    let mut reader = Cursor::new(input);
    let mut writer = Cursor::new(Vec::new());

    interpret(&mut reader, &mut writer, &program_string).unwrap();
    assert_eq!(std::str::from_utf8(&writer.into_inner()).unwrap(), output);
}
