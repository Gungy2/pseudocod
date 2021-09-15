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
fn integration_test(file_name: &str, out: &str, stdin: &str) {
    let mut cmd = assert_cmd::Command::cargo_bin("pseudocod").unwrap();
    let path = std::path::Path::new("tests")
        .join("resources")
        .join(file_name);
    let file = path.to_str().unwrap();

    let assert = cmd.arg(file).write_stdin(stdin).assert();
    assert
        .success()
        .stdout(format!("Se executa...\n{}Gata!\n", out));
}
