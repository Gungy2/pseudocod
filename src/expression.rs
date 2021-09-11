use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::{
        complete::{char, digit1 as digit, space0 as space},
        is_alphabetic, is_alphanumeric,
    },
    combinator::{fail, map, map_res},
    multi::fold_many0,
    sequence::{delimited, pair},
    IResult,
};
use std::num::ParseIntError;

#[derive(PartialEq, Clone, Debug)]
pub enum Expression<'a> {
    Constant(i32),
    Variable(&'a str),
    Multiplication(Box<Expression<'a>>, Box<Expression<'a>>),
    Division(Box<Expression<'a>>, Box<Expression<'a>>),
    Addition(Box<Expression<'a>>, Box<Expression<'a>>),
    Subtraction(Box<Expression<'a>>, Box<Expression<'a>>),
}

fn parens(i: &str) -> IResult<&str, Expression> {
    delimited(space, delimited(tag("("), expr, tag(")")), space)(i)
}

fn factor(i: &str) -> IResult<&str, Expression> {
    alt((
        map_res::<_, _, _, _, ParseIntError, _, _>(
            delimited(space, digit, space),
            |num_str: &str| Ok(Expression::Constant(num_str.parse()?)),
        ),
        map(delimited(space, id, space), |id: &str| {
            Expression::Variable(id)
        }),
        parens,
    ))(i)
}

fn id(i: &str) -> IResult<&str, &str> {
    if !is_alphabetic(i.chars().next().unwrap() as u8) {
        return fail(i);
    }
    take_while(|c| is_alphanumeric(c as u8) || (c as char == '_'))(i)
}

fn term(i: &str) -> IResult<&str, Expression> {
    let (i, init) = factor(i)?;

    fold_many0(
        pair(alt((char('*'), char('/'))), factor),
        move || init.clone(),
        |acc, (op, expr)| {
            if op == '*' {
                Expression::Multiplication(Box::new(acc), Box::new(expr))
            } else {
                Expression::Division(Box::new(acc), Box::new(expr))
            }
        },
    )(i)
}

pub fn expr(i: &str) -> IResult<&str, Expression> {
    let (i, init) = term(i)?;

    fold_many0(
        pair(alt((char('+'), char('-'))), term),
        move || init.clone(),
        |acc, (op, expr)| {
            if op == '+' {
                Expression::Addition(Box::new(acc), Box::new(expr))
            } else {
                Expression::Subtraction(Box::new(acc), Box::new(expr))
            }
        },
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn factor_test() {
        assert_eq!(factor("3"), Ok(("", Expression::Constant(3))));
        assert_eq!(factor(" 12"), Ok(("", Expression::Constant(12))));
        assert_eq!(factor("537  "), Ok(("", Expression::Constant(537))));
        assert_eq!(factor("  24   "), Ok(("", Expression::Constant(24))));
        assert_eq!(factor("a"), Ok(("", Expression::Variable("a"))));
        assert_eq!(factor(" as3234"), Ok(("", Expression::Variable("as3234"))));
        assert_eq!(
            factor("variable_name  "),
            Ok(("", Expression::Variable("variable_name")))
        );
    }

    #[test]
    fn term_test() {
        assert_eq!(
            term(" 12 *2 /  3"),
            Ok((
                "",
                Expression::Division(
                    Box::new(Expression::Multiplication(
                        Box::new(Expression::Constant(12)),
                        Box::new(Expression::Constant(2))
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );

        assert_eq!(
            term(" 2* 3  *2 *2 /  3"),
            Ok((
                "",
                Expression::Division(
                    Box::new(Expression::Multiplication(
                        Box::new(Expression::Multiplication(
                            Box::new(Expression::Multiplication(
                                Box::new(Expression::Constant(2)),
                                Box::new(Expression::Constant(3))
                            )),
                            Box::new(Expression::Constant(2))
                        )),
                        Box::new(Expression::Constant(2))
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );
        assert_eq!(
            term(" 48 /  3/2"),
            Ok((
                "",
                Expression::Division(
                    Box::new(Expression::Division(
                        Box::new(Expression::Constant(48)),
                        Box::new(Expression::Constant(3))
                    )),
                    Box::new(Expression::Constant(2)),
                )
            ))
        );
    }

    #[test]
    fn expr_test() {
        assert_eq!(
            expr(" 1 +  2 "),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Constant(1)),
                    Box::new(Expression::Constant(2)),
                )
            ))
        );
        assert_eq!(
            expr(" 12 + 6 - 4+  3"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Subtraction(
                        Box::new(Expression::Addition(
                            Box::new(Expression::Constant(12)),
                            Box::new(Expression::Constant(6))
                        )),
                        Box::new(Expression::Constant(4))
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );
        assert_eq!(
            expr(" 1 + 2*3 + 4"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Addition(
                        Box::new(Expression::Constant(1)),
                        Box::new(Expression::Multiplication(
                            Box::new(Expression::Constant(2)),
                            Box::new(Expression::Constant(3))
                        )),
                    ),),
                    Box::new(Expression::Constant(4)),
                )
            ))
        );
    }

    #[test]
    fn parens_test() {
        assert_eq!(expr(" (  2 )"), Ok(("", Expression::Constant(2))));
        assert_eq!(
            expr(" 2* (  3 + 4 ) "),
            Ok((
                "",
                Expression::Multiplication(
                    Box::new(Expression::Constant(2)),
                    Box::new(Expression::Addition(
                        Box::new(Expression::Constant(3)),
                        Box::new(Expression::Constant(4))
                    ))
                )
            ))
        );
        assert_eq!(
            expr("  2*2 / ( 5 - 1) + 3"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Division(
                        Box::new(Expression::Multiplication(
                            Box::new(Expression::Constant(2)),
                            Box::new(Expression::Constant(2))
                        )),
                        Box::new(Expression::Subtraction(
                            Box::new(Expression::Constant(5)),
                            Box::new(Expression::Constant(1))
                        )),
                    )),
                    Box::new(Expression::Constant(3))
                )
            ))
        );
        assert_eq!(
            expr("  var - 2 / ( 5 - c) + 3"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Subtraction(
                        Box::new(Expression::Variable("var")),
                        Box::new(Expression::Division(
                            Box::new(Expression::Constant(2)),
                            Box::new(Expression::Subtraction(
                                Box::new(Expression::Constant(5)),
                                Box::new(Expression::Variable("c"))
                            )),
                        )),
                    )),
                    Box::new(Expression::Constant(3))
                )
            ))
        );
    }
}
