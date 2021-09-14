use super::expression::{expr, id, Expression};
use nom::character::complete::multispace0;
use nom::combinator::eof;
use nom::error::ParseError;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, space0, space1},
    combinator::{map, opt},
    multi::{count, many1, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

pub type Block<'a> = Vec<Instruction<'a>>;

#[derive(PartialEq, Clone, Debug)]
pub enum WhileType {
    While,
    DoWhile,
    Repeat,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Instruction<'a> {
    Read(Vec<&'a str>),
    Write(Expression<'a>),
    Assignment(&'a str, Expression<'a>),
    If(Expression<'a>, Block<'a>, Option<Block<'a>>),
    While(WhileType, Expression<'a>, Block<'a>),
    For {
        variable: &'a str,
        start_expr: Expression<'a>,
        end_expr: Expression<'a>,
        step: Expression<'a>,
        block: Block<'a>,
    },
}

fn read<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Instruction, E> {
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

fn write<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Instruction, E> {
    map(
        preceded(delimited(space0, tag("scrie"), space1), expr),
        Instruction::Write,
    )(i)
}

fn assignment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Instruction, E> {
    map(
        pair(
            terminated(preceded(space0, id), delimited(space0, tag("<-"), space0)),
            expr,
        ),
        |(id, expr)| Instruction::Assignment(id, expr),
    )(i)
}

fn instruction<'a, E: ParseError<&'a str>>(
    indent: usize,
) -> impl Fn(&'a str) -> IResult<&str, Instruction<'a>, E> {
    move |i: &'a str| {
        dbg!(&i);
        alt((
            read,
            write,
            assignment,
            if_instr(indent),
            while_instr(indent),
            do_while_instr(indent),
            repeat_instr(indent),
            for_instr(indent),
        ))(i)
    }
}

fn if_instr<'a, E: ParseError<&'a str>>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, Instruction<'a>, E> {
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

fn while_instr<'a, E: ParseError<&'a str>>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, Instruction<'a>, E> {
    map(
        pair(
            preceded(tuple((tag("cat"), space1, tag("timp"), space1)), expr),
            preceded(terminated(tag("executa"), space0), block(indent + 1)),
        ),
        |(expr, block)| Instruction::While(WhileType::While, expr, block),
    )
}

fn do_while_instr<'a, E: ParseError<&'a str>>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, Instruction<'a>, E> {
    map(
        pair(
            preceded(terminated(tag("executa"), space0), block(indent + 1)),
            preceded(
                tuple((indentation(indent), tag("cat"), space1, tag("timp"), space1)),
                expr,
            ),
        ),
        |(block, expr)| Instruction::While(WhileType::DoWhile, expr, block),
    )
}

fn repeat_instr<'a, E: ParseError<&'a str>>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, Instruction<'a>, E> {
    map(
        pair(
            preceded(terminated(tag("repeta"), space0), block(indent + 1)),
            preceded(
                tuple((
                    indentation(indent),
                    tag("pana"),
                    space1,
                    tag("cand"),
                    space1,
                )),
                expr,
            ),
        ),
        |(block, expr)| Instruction::While(WhileType::Repeat, expr, block),
    )
}

fn for_instr<'a, E: ParseError<&'a str>>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, Instruction<'a>, E> {
    map(
        tuple((
            preceded(tuple((tag("pentru"), space1)), assignment),
            preceded(delimited(space0, char(','), space0), expr),
            opt(preceded(delimited(space0, char(','), space0), expr)),
            preceded(terminated(tag("executa"), space0), block(indent + 1)),
        )),
        |(assignment, end_expr, step, block)| {
            let step = step.unwrap_or(Expression::Constant(1));
            if let Instruction::Assignment(variable, start_expr) = assignment {
                Instruction::For {
                    variable,
                    start_expr,
                    end_expr,
                    step,
                    block,
                }
            } else {
                panic!("Invalid assignment in for loop!")
            }
        },
    )
}

fn block<'a, E: ParseError<&'a str>>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&'a str, Block<'a>, E> {
    many1(preceded(indentation(indent), instruction(indent)))
}

pub fn program<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Block, E> {
    terminated(block(0), multispace0)(i)
}

fn indentation<'a, E: ParseError<&'a str>>(
    indent: usize,
) -> impl FnMut(&'a str) -> IResult<&str, (), E> {
    map(pair(line_ending, count(char(' '), 2 * indent)), |_| ())
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::error::Error;

    #[test]
    fn read_test() {
        assert_eq!(
            read::<Error<&str>>("citeste a  "),
            Ok(("", Instruction::Read(vec!["a"])))
        );
        assert_eq!(
            read::<Error<&str>>("  citeste   a  , b   "),
            Ok(("", Instruction::Read(vec!["a", "b"])))
        );
        assert_eq!(
            read::<Error<&str>>(" citeste   a34  , b123,a_34   "),
            Ok(("", Instruction::Read(vec!["a34", "b123", "a_34"])))
        );
    }

    #[test]
    fn write_test() {
        assert_eq!(
            write::<Error<&str>>("scrie 1"),
            Ok(("", Instruction::Write(Expression::Constant(1))))
        );
        assert_eq!(
            write::<Error<&str>>("  scrie  6 + 5  "),
            Ok((
                "",
                Instruction::Write(Expression::Addition(
                    Box::new(Expression::Constant(6)),
                    Box::new(Expression::Constant(5)),
                ))
            ))
        );
        assert_eq!(
            write::<Error<&str>>(" scrie (a + b) - 3"),
            Ok((
                "",
                Instruction::Write(Expression::Subtraction(
                    Box::new(Expression::Addition(
                        Box::new(Expression::Variable("a")),
                        Box::new(Expression::Variable("b")),
                    )),
                    Box::new(Expression::Constant(3)),
                ))
            ))
        );
        assert_eq!(
            write::<Error<&str>>("scrie 3 * 4 + var"),
            Ok((
                "",
                Instruction::Write(Expression::Addition(
                    Box::new(Expression::Multiplication(
                        Box::new(Expression::Constant(3)),
                        Box::new(Expression::Constant(4)),
                    )),
                    Box::new(Expression::Variable("var")),
                ))
            ))
        );
    }

    #[test]
    fn assignment_test() {
        assert_eq!(
            assignment::<Error<&str>>("x <- 1"),
            Ok(("", Instruction::Assignment("x", Expression::Constant(1))))
        );
        assert_eq!(
            assignment::<Error<&str>>(" var12<-6 + 5 "),
            Ok((
                "",
                Instruction::Assignment(
                    "var12",
                    Expression::Addition(
                        Box::new(Expression::Constant(6)),
                        Box::new(Expression::Constant(5)),
                    ),
                )
            ))
        );
        assert_eq!(
            assignment::<Error<&str>>(" v <- 5 + x * y "),
            Ok((
                "",
                Instruction::Assignment(
                    "v",
                    Expression::Addition(
                        Box::new(Expression::Constant(5)),
                        Box::new(Expression::Multiplication(
                            Box::new(Expression::Variable("x")),
                            Box::new(Expression::Variable("y")),
                        )),
                    ),
                )
            ))
        );
    }

    #[test]
    fn if_test() {
        assert_eq!(
            if_instr::<Error<&str>>(0)("daca 1 atunci \n  scrie 15"),
            Ok((
                "",
                Instruction::If(
                    Expression::Constant(1),
                    vec![Instruction::Write(Expression::Constant(15))],
                    None,
                )
            ))
        );
        assert_eq!(
            if_instr::<Error<&str>>(0)("daca 5 + 5 atunci \n  scrie 10\n  scrie 16"),
            Ok((
                "",
                Instruction::If(
                    Expression::Addition(
                        Box::new(Expression::Constant(5)),
                        Box::new(Expression::Constant(5)),
                    ),
                    vec![
                        Instruction::Write(Expression::Constant(10)),
                        Instruction::Write(Expression::Constant(16)),
                    ],
                    None,
                )
            ))
        );
        assert_eq!(
            if_instr::<Error<&str>>(0)("daca 1 atunci\n  daca 2 atunci\n    scrie 5\n    scrie 6"),
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
                        None,
                    )],
                    None,
                )
            ))
        );
        assert_eq!(
            if_instr::<Error<&str>>(0)("daca 1 atunci\n  daca 2 atunci\n    scrie 5\n  scrie 6"),
            Ok((
                "",
                Instruction::If(
                    Expression::Constant(1),
                    vec![
                        Instruction::If(
                            Expression::Constant(2),
                            vec![Instruction::Write(Expression::Constant(5))],
                            None,
                        ),
                        Instruction::Write(Expression::Constant(6)),
                    ],
                    None,
                )
            ))
        );
        assert_eq!(
            if_instr::<Error<&str>>(0)(
                "daca 1 atunci\n  daca 2 atunci\n    scrie 5\n  scrie 6\naltfel\n  scrie 1"
            ),
            Ok((
                "",
                Instruction::If(
                    Expression::Constant(1),
                    vec![
                        Instruction::If(
                            Expression::Constant(2),
                            vec![Instruction::Write(Expression::Constant(5))],
                            None,
                        ),
                        Instruction::Write(Expression::Constant(6)),
                    ],
                    Some(vec![Instruction::Write(Expression::Constant(1))]),
                )
            ))
        );
    }

    #[test]
    fn while_test() {
        assert_eq!(
            while_instr::<Error<&str>>(0)("cat timp 1 executa\n  scrie 2\n  scrie 4"),
            Ok((
                "",
                Instruction::While(
                    WhileType::While,
                    Expression::Constant(1),
                    vec![
                        Instruction::Write(Expression::Constant(2)),
                        Instruction::Write(Expression::Constant(4)),
                    ],
                )
            ))
        );
        assert_eq!(
            while_instr::<Error<&str>>(0)("cat timp 1 executa\n  cat timp 2 executa\n    scrie 1"),
            Ok((
                "",
                Instruction::While(
                    WhileType::While,
                    Expression::Constant(1),
                    vec![Instruction::While(
                        WhileType::While,
                        Expression::Constant(2),
                        vec![Instruction::Write(Expression::Constant(1))],
                    )],
                )
            ))
        );
    }

    #[test]
    fn do_while_test() {
        assert_eq!(
            do_while_instr::<Error<&str>>(0)("executa\n  scrie x\n  scrie 2\ncat timp 1"),
            Ok((
                "",
                Instruction::While(
                    WhileType::DoWhile,
                    Expression::Constant(1),
                    vec![
                        Instruction::Write(Expression::Variable("x")),
                        Instruction::Write(Expression::Constant(2)),
                    ],
                )
            ))
        );
        assert_eq!(
            do_while_instr::<Error<&str>>(0)(
                "executa\n  executa\n    scrie x\n  cat timp m\ncat timp 1"
            ),
            Ok((
                "",
                Instruction::While(
                    WhileType::DoWhile,
                    Expression::Constant(1),
                    vec![Instruction::While(
                        WhileType::DoWhile,
                        Expression::Variable("m"),
                        vec![Instruction::Write(Expression::Variable("x"))],
                    )],
                )
            ))
        );
    }

    #[test]
    fn repeat_test() {
        assert_eq!(
            repeat_instr::<Error<&str>>(0)("repeta\n  scrie x\n  scrie 5\npana cand 1"),
            Ok((
                "",
                Instruction::While(
                    WhileType::Repeat,
                    Expression::Constant(1),
                    vec![
                        Instruction::Write(Expression::Variable("x")),
                        Instruction::Write(Expression::Constant(5)),
                    ],
                )
            ))
        );
        assert_eq!(
            repeat_instr::<Error<&str>>(0)(
                "repeta\n  repeta\n    scrie 13\n  pana cand m\npana cand 1"
            ),
            Ok((
                "",
                Instruction::While(
                    WhileType::Repeat,
                    Expression::Constant(1),
                    vec![Instruction::While(
                        WhileType::Repeat,
                        Expression::Variable("m"),
                        vec![Instruction::Write(Expression::Constant(13))],
                    )],
                )
            ))
        );
    }

    #[test]
    fn for_test() {
        assert_eq!(
            for_instr::<Error<&str>>(0)("pentru x<-1, 2 executa\n  scrie x"),
            Ok((
                "",
                Instruction::For {
                    variable: "x",
                    start_expr: Expression::Constant(1),
                    end_expr: Expression::Constant(2),
                    step: Expression::Constant(1),
                    block: vec![Instruction::Write(Expression::Variable("x"))],
                }
            ))
        );
        assert_eq!(
            for_instr::<Error<&str>>(0)("pentru var<- 0, 5  , 2   executa\n  scrie var"),
            Ok((
                "",
                Instruction::For {
                    variable: "var",
                    start_expr: Expression::Constant(0),
                    end_expr: Expression::Constant(5),
                    step: Expression::Constant(2),
                    block: vec![Instruction::Write(Expression::Variable("var"))],
                }
            ))
        );
    }
}
