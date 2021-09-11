use super::expression::id;
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending, multispace0, space0, space1},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, terminated},
    IResult,
};

pub type Program<'a> = Vec<Instruction<'a>>;

#[derive(PartialEq, Clone, Debug)]
pub enum Instruction<'a> {
    Read(Vec<&'a str>),
}

fn read(i: &str) -> IResult<&str, Instruction> {
    dbg!(i);
    map(
        preceded(
            delimited(space0, tag("citeste"), space1),
            terminated(
                separated_list1(delimited(space0, char(','), space0), id),
                space0,
            ),
        ),
        Instruction::Read,
    )(i)
}

fn instruction(i: &str) -> IResult<&str, Instruction> {
    read(i)
}

pub fn program(i: &str) -> IResult<&str, Program> {
    delimited(
        multispace0,
        separated_list1(many1(delimited(space0, line_ending, space0)), instruction),
        multispace0,
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::Path;
    use test_case::test_case;

    #[test]
    fn read_test() {
        assert_eq!(read("citeste a  "), Ok(("", Instruction::Read(vec!["a"]))));
        assert_eq!(
            read("  citeste   a  , b   "),
            Ok(("", Instruction::Read(vec!["a", "b"])))
        );
        assert_eq!(
            read(" citeste   a34  , b123,a_34   "),
            Ok(("", Instruction::Read(vec!["a34", "b123", "a_34"])))
        );
    }

    #[test_case("reads.pseudocode", 
        vec![
            Instruction::Read(vec!["a", "b"]), 
            Instruction::Read(vec!["a", "c", "d"])] 
    ; "simple read program")]

    fn program_test(path: &str, result: Program) {
        assert_eq!(
            program(
                &fs::read_to_string(Path::new("tests/resources").join(path))
                    .expect("Invalid path to test file")
            ),
            Ok(("", result))
        );
    }
}
