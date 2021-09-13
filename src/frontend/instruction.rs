use super::expression::{expr, id, Expression};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, multispace0, space0, space1},
    combinator::{map, opt},
    multi::{count, many1, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

pub type Block<'a> = Vec<Instruction<'a>>;

#[derive(PartialEq, Clone, Debug)]
pub enum Instruction<'a> {
    Read(Vec<&'a str>),
    Write(Expression<'a>),
    Assignment(&'a str, Expression<'a>),
    If(Expression<'a>, Block<'a>, Option<Block<'a>>),
}

fn read(i: &str) -> IResult<&str, Instruction> {
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

fn write(i: &str) -> IResult<&str, Instruction> {
    dbg!(i);
    map(
        preceded(delimited(space0, tag("scrie"), space1), expr),
        Instruction::Write,
    )(i)
}

fn assignment(i: &str) -> IResult<&str, Instruction> {
    map(
        pair(
            terminated(preceded(space0, id), delimited(space0, tag("<-"), space0)),
            expr,
        ),
        |(id, expr)| Instruction::Assignment(id, expr),
    )(i)
}

fn instruction<'a>(
    indent: usize,
) -> impl Fn(&'a str) -> IResult<&str, Instruction<'a>, nom::error::Error<&'a str>> {
    move |i: &'a str| alt((read, write, assignment, if_instr(indent)))(i)
}

fn if_instr<'a>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, Instruction<'a>, nom::error::Error<&'a str>> {
    map(
        tuple((
            preceded(terminated(tag("daca"), space0), expr),
            preceded(terminated(tag("atunci"), space0), block(indent + 1)),
            opt(preceded(
                terminated(pair(indentation(indent), tag("altfel")), space0),
                block(indent + 1),
            )),
        )),
        |(expr, if_block, else_block)| Instruction::If(expr, if_block, else_block),
    )
}

fn block<'a>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, Block<'a>, nom::error::Error<&'a str>> {
    many1(preceded(indentation(indent), instruction(indent)))
}

pub fn program(i: &str) -> IResult<&str, Block> {
    block(0)(i)
}

fn indentation<'a>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, (), nom::error::Error<&'a str>> {
    map(pair(line_ending, count(char(' '), 2 * indent)), |_| ())
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

    #[test]
    fn if_test() {
        assert_eq!(
            if_instr(0)("daca 1 atunci \n  scrie 15"),
            Ok((
                "",
                Instruction::If(
                    Expression::Constant(1),
                    vec![Instruction::Write(Expression::Constant(15))],
                    None
                )
            ))
        );
        assert_eq!(
            if_instr(0)("daca 5 + 5 atunci \n  scrie 10\n  scrie 16"),
            Ok((
                "",
                Instruction::If(
                    Expression::Addition(
                        Box::new(Expression::Constant(5)),
                        Box::new(Expression::Constant(5))
                    ),
                    vec![
                        Instruction::Write(Expression::Constant(10)),
                        Instruction::Write(Expression::Constant(16))
                    ],
                    None
                )
            ))
        );
        assert_eq!(
            if_instr(0)("daca 1 atunci\n  daca 2 atunci\n    scrie 5\n    scrie 6"),
            Ok((
                "",
                Instruction::If(
                    Expression::Constant(1),
                    vec![Instruction::If(
                        Expression::Constant(2),
                        vec![
                            Instruction::Write(Expression::Constant(5)),
                            Instruction::Write(Expression::Constant(6)),
                        ],
                        None
                    )],
                    None
                )
            ))
        );
        assert_eq!(
            if_instr(0)("daca 1 atunci\n  daca 2 atunci\n    scrie 5\n  scrie 6"),
            Ok((
                "",
                Instruction::If(
                    Expression::Constant(1),
                    vec![
                        Instruction::If(
                            Expression::Constant(2),
                            vec![Instruction::Write(Expression::Constant(5)),],
                            None
                        ),
                        Instruction::Write(Expression::Constant(6)),
                    ],
                    None
                )
            ))
        );
    }

    //     #[test_case("reads.pseudo",
    //         vec![
    //             Instruction::Read(vec!["a", "b"]),
    //             Instruction::Read(vec!["a", "c", "d"])]
    //     ; "simple read program")]
    //     fn program_test(path: &str, result: Block) {
    //         assert_eq!(
    //             program(
    //                 &fs::read_to_string(Path::new("tests/resources").join(path))
    //                     .expect("Invalid path to test file")
    //             ),
    //             Ok(("", result))
    //         );
    //     }
}
