use nom::error::ParseError;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::{
        complete::{char, digit1, space0},
        is_alphabetic, is_alphanumeric,
    },
    combinator::{fail, map},
    multi::fold_many0,
    sequence::{delimited, pair},
    IResult,
};

#[derive(PartialEq, Clone, Debug)]
pub enum Expression<'a> {
    Constant(u32),
    Variable(&'a str),
    Multiplication(Box<Expression<'a>>, Box<Expression<'a>>),
    Division(Box<Expression<'a>>, Box<Expression<'a>>),
    Addition(Box<Expression<'a>>, Box<Expression<'a>>),
    Subtraction(Box<Expression<'a>>, Box<Expression<'a>>),
    Reminder(Box<Expression<'a>>, Box<Expression<'a>>),
    Minus(Box<Expression<'a>>),
    Order(OrderType, Box<Expression<'a>>, Box<Expression<'a>>),
}

#[derive(PartialEq, Clone, Debug)]
pub enum OrderType {
    Less,
    LessOrEqual,
    Equal,
    Greater,
    GreaterOrEqual,
}

impl From<&str> for OrderType {
    fn from(symbol: &str) -> Self {
        match symbol {
            "<" => OrderType::Less,
            "<=" => OrderType::LessOrEqual,
            "=" => OrderType::Equal,
            ">=" => OrderType::GreaterOrEqual,
            ">" => OrderType::Greater,
            _ => panic!("Operator invalid!"),
        }
    }
}

fn parens<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Expression, E> {
    delimited(space0, delimited(tag("("), expr, tag(")")), space0)(i)
}

fn factor<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Expression, E> {
    alt((
        map(delimited(space0, digit1, space0), |num_str: &str| {
            Expression::Constant(num_str.parse().unwrap())
        }),
        map(delimited(space0, id, space0), |id: &str| {
            Expression::Variable(id)
        }),
        parens,
    ))(i)
}

pub fn id<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    if let Some(chr) = i.chars().next() {
        if !is_alphabetic(chr as u8) {
            return fail(i);
        }
    } else {
        return fail(i);
    }
    take_while(|c| is_alphanumeric(c as u8) || (c as char == '_'))(i)
}

fn term<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Expression, E> {
    let (i, init) = factor(i)?;

    fold_many0(
        pair(alt((char('*'), char('/'), char('%'))), factor),
        move || init.clone(),
        |acc, (op, expr)| match op {
            '*' => Expression::Multiplication(Box::new(acc), Box::new(expr)),
            '/' => Expression::Division(Box::new(acc), Box::new(expr)),
            _ => Expression::Reminder(Box::new(acc), Box::new(expr)),
        },
    )(i)
}

fn member<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Expression, E> {
    if let Ok((i, init)) = term::<'a, E>(i) {
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
    } else {
        let (i, _) = delimited(space0, char('-'), space0)(i)?;
        map(term, |expr| Expression::Minus(Box::new(expr)))(i)
    }
}

pub fn expr<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Expression, E> {
    let (i, init) = member(i)?;

    fold_many0(
        pair(
            alt((tag("<="), tag(">="), tag("<"), tag(">"), tag("="))),
            member,
        ),
        move || init.clone(),
        |acc, (op, expr)| Expression::Order(OrderType::from(op), Box::new(acc), Box::new(expr)),
    )(i)
}

#[cfg(test)]
mod test {
    use nom::combinator::eof;
    use super::*;
    use nom::error::Error;

    #[test]
    fn factor_test() {
        assert_eq!(
            factor::<Error<&str>>("3"),
            Ok(("", Expression::Constant(3)))
        );
        assert_eq!(
            factor::<Error<&str>>(" 12"),
            Ok(("", Expression::Constant(12)))
        );
        assert_eq!(
            factor::<Error<&str>>("537  "),
            Ok(("", Expression::Constant(537)))
        );
        assert_eq!(
            factor::<Error<&str>>("  24   "),
            Ok(("", Expression::Constant(24)))
        );
        assert_eq!(
            factor::<Error<&str>>("a"),
            Ok(("", Expression::Variable("a")))
        );
        assert_eq!(
            factor::<Error<&str>>(" as3234"),
            Ok(("", Expression::Variable("as3234")))
        );
        assert_eq!(
            factor::<Error<&str>>("variable_name  "),
            Ok(("", Expression::Variable("variable_name")))
        );
    }

    #[test]
    fn term_test() {
        assert_eq!(
            term::<Error<&str>>(" 12 *2 /  3"),
            Ok((
                "",
                Expression::Division(
                    Box::new(Expression::Multiplication(
                        Box::new(Expression::Constant(12)),
                        Box::new(Expression::Constant(2)),
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );

        assert_eq!(
            term::<Error<&str>>(" 2* 3  *2 *2 /  3"),
            Ok((
                "",
                Expression::Division(
                    Box::new(Expression::Multiplication(
                        Box::new(Expression::Multiplication(
                            Box::new(Expression::Multiplication(
                                Box::new(Expression::Constant(2)),
                                Box::new(Expression::Constant(3)),
                            )),
                            Box::new(Expression::Constant(2)),
                        )),
                        Box::new(Expression::Constant(2)),
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );
        assert_eq!(
            term::<Error<&str>>(" 48 /  3/2"),
            Ok((
                "",
                Expression::Division(
                    Box::new(Expression::Division(
                        Box::new(Expression::Constant(48)),
                        Box::new(Expression::Constant(3)),
                    )),
                    Box::new(Expression::Constant(2)),
                )
            ))
        );
    }

    #[test]
    fn member_test() {
        assert_eq!(
            member::<Error<&str>>(" 1 +  2 "),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Constant(1)),
                    Box::new(Expression::Constant(2)),
                )
            ))
        );
        assert_eq!(
            member::<Error<&str>>(" 12 + 6 - 4+  3"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Subtraction(
                        Box::new(Expression::Addition(
                            Box::new(Expression::Constant(12)),
                            Box::new(Expression::Constant(6)),
                        )),
                        Box::new(Expression::Constant(4)),
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );
        assert_eq!(
            member::<Error<&str>>(" 1 + 2*3 + 4"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Addition(
                        Box::new(Expression::Constant(1)),
                        Box::new(Expression::Multiplication(
                            Box::new(Expression::Constant(2)),
                            Box::new(Expression::Constant(3)),
                        )),
                    ),),
                    Box::new(Expression::Constant(4)),
                )
            ))
        );
    }

    #[test]
    fn parens_test() {
        assert_eq!(
            expr::<Error<&str>>(" (  2 )"),
            Ok(("", Expression::Constant(2)))
        );
        assert_eq!(
            expr::<Error<&str>>(" ( -52 )"),
            Ok(("", Expression::Minus(Box::new(Expression::Constant(52)))))
        );
        assert_eq!(
            expr::<Error<&str>>(" ( -var )"),
            Ok(("", Expression::Minus(Box::new(Expression::Variable("var")))))
        );
        assert_eq!(
            expr::<Error<&str>>(" (a +  b ) - 5"),
            Ok((
                "",
                Expression::Subtraction(
                    Box::new(Expression::Addition(
                        Box::new(Expression::Variable("a")),
                        Box::new(Expression::Variable("b")),
                    )),
                    Box::new(Expression::Constant(5)),
                )
            ))
        );
        assert_eq!(
            expr::<Error<&str>>(" 4 * ( - var_name )  + 6"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Multiplication(
                        Box::new(Expression::Constant(4)),
                        Box::new(Expression::Minus(Box::new(Expression::Variable(
                            "var_name"
                        )))),
                    )),
                    Box::new(Expression::Constant(6)),
                )
            ))
        );
        assert_eq!(
            expr::<Error<&str>>(" 2* (  3 + 4 ) "),
            Ok((
                "",
                Expression::Multiplication(
                    Box::new(Expression::Constant(2)),
                    Box::new(Expression::Addition(
                        Box::new(Expression::Constant(3)),
                        Box::new(Expression::Constant(4)),
                    )),
                )
            ))
        );
        assert_eq!(
            expr::<Error<&str>>("  2*2 / ( 5 - 1) + 3"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Division(
                        Box::new(Expression::Multiplication(
                            Box::new(Expression::Constant(2)),
                            Box::new(Expression::Constant(2)),
                        )),
                        Box::new(Expression::Subtraction(
                            Box::new(Expression::Constant(5)),
                            Box::new(Expression::Constant(1)),
                        )),
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );
        assert_eq!(
            expr::<Error<&str>>("  var - 2 / ( 5 - c) + 3"),
            Ok((
                "",
                Expression::Addition(
                    Box::new(Expression::Subtraction(
                        Box::new(Expression::Variable("var")),
                        Box::new(Expression::Division(
                            Box::new(Expression::Constant(2)),
                            Box::new(Expression::Subtraction(
                                Box::new(Expression::Constant(5)),
                                Box::new(Expression::Variable("c")),
                            )),
                        )),
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );
        assert!(expr::<Error<&str>>("(1 + 4 - 4()").is_err());
    }

    #[test]
    fn expr_test() {
        assert_eq!(
            expr::<Error<&str>>("5 < 3"),
            Ok((
                "",
                Expression::Order(
                    OrderType::Less,
                    Box::new(Expression::Constant(5)),
                    Box::new(Expression::Constant(3))
                )
            ))
        );
        assert_eq!(
            expr::<Error<&str>>("x <= 6 + 3"),
            Ok((
                "",
                Expression::Order(
                    OrderType::LessOrEqual,
                    Box::new(Expression::Variable("x")),
                    Box::new(Expression::Addition(
                        Box::new(Expression::Constant(6)),
                        Box::new(Expression::Constant(3))
                    ))
                )
            ))
        );
        assert_eq!(
            expr::<Error<&str>>("(x = 6) > 3"),
            Ok((
                "",
                Expression::Order(
                    OrderType::Greater,
                    Box::new(Expression::Order(
                        OrderType::Equal,
                        Box::new(Expression::Variable("x")),
                        Box::new(Expression::Constant(6))
                    )),
                    Box::new(Expression::Constant(3)),
                )
            ))
        );
        assert!(pair(expr::<Error<&str>>, eof)("2 >").is_err());
    }
}
