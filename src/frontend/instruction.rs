use super::expression::{self, Expression};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, multispace0, space0, space1},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

pub type Program<'a> = Vec<Instruction<'a>>;

#[derive(PartialEq, Clone, Debug)]
pub enum Instruction<'a> {
    Read(Vec<&'a str>),
    Write(Expression<'a>),
    Assignment(&'a str, Expression<'a>),
}

fn read(i: &str) -> IResult<&str, Instruction> {
    map(
        preceded(
            delimited(space0, tag("citeste"), space1),
            terminated(
                separated_list1(delimited(space0, char(','), space0), expression::id),
                space0,
            ),
        ),
        Instruction::Read,
    )(i)
}

fn write(i: &str) -> IResult<&str, Instruction> {
    map(
        preceded(delimited(space0, tag("scrie"), space1), expression::expr),
        Instruction::Write,
    )(i)
}

fn assignment(i: &str) -> IResult<&str, Instruction> {
    map(
        pair(
            terminated(
                preceded(space0, expression::id),
                delimited(space0, tag("<-"), space0),
            ),
            expression::expr,
        ),
        |(id, expr)| Instruction::Assignment(id, expr),
    )(i)
}

fn instruction(i: &str) -> IResult<&str, Instruction> {
    alt((read, write, assignment))(i)
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

    #[test]
    fn write_test() {
        assert_eq!(
            write("scrie 1"),
            Ok(("", Instruction::Write(Expression::Constant(1))))
        );
        assert_eq!(
            write("  scrie  6 + 5  "),
            Ok((
                "",
                Instruction::Write(Expression::Addition(
                    Box::new(Expression::Constant(6)),
                    Box::new(Expression::Constant(5))
                ))
            ))
        );
        assert_eq!(
            write(" scrie (a + b) - 3"),
            Ok((
                "",
                Instruction::Write(Expression::Subtraction(
                    Box::new(Expression::Addition(
                        Box::new(Expression::Variable("a")),
                        Box::new(Expression::Variable("b"))
                    )),
                    Box::new(Expression::Constant(3))
                ))
            ))
        );
        assert_eq!(
            write("scrie 3 * 4 + var"),
            Ok((
                "",
                Instruction::Write(Expression::Addition(
                    Box::new(Expression::Multiplication(
                        Box::new(Expression::Constant(3)),
                        Box::new(Expression::Constant(4))
                    )),
                    Box::new(Expression::Variable("var"))
                ))
            ))
        );
    }

    #[test]
    fn assignment_test() {
        assert_eq!(
            assignment("x <- 1"),
            Ok(("", Instruction::Assignment("x", Expression::Constant(1))))
        );
        assert_eq!(
            assignment(" var12<-6 + 5 "),
            Ok((
                "",
                Instruction::Assignment(
                    "var12",
                    Expression::Addition(
                        Box::new(Expression::Constant(6)),
                        Box::new(Expression::Constant(5))
                    )
                )
            ))
        );
        assert_eq!(
            assignment(" v <- 5 + x * y "),
            Ok((
                "",
                Instruction::Assignment(
                    "v",
                    Expression::Addition(
                        Box::new(Expression::Constant(5)),
                        Box::new(Expression::Multiplication(
                            Box::new(Expression::Variable("x")),
                            Box::new(Expression::Variable("y"))
                        ))
                    )
                )
            ))
        );
    }

    #[test_case("reads.pseudo", 
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
