use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::{
        complete::{digit1 as digit, space0 as space},
        is_alphabetic, is_alphanumeric,
    },
    combinator::{fail, map, map_res},
    sequence::delimited,
    IResult,
};
use std::{num::ParseIntError, str::FromStr};

#[derive(PartialEq, Clone, Debug)]
pub enum Expression<'a> {
    Constant(i32),
    Variable(&'a str),
    Bracket(Box<Expression<'a>>),
}

fn parens(i: &str) -> IResult<&str, Expression> {
    delimited(space, delimited(tag("("), factor, tag(")")), space)(i)
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

// // We read an initial factor and for each time we find
// // a * or / operator followed by another factor, we do
// // the math by folding everything
// fn term(i: &str) -> IResult<&str, Expression> {
//     let (i, init) = factor(i)?;

//     fold_many0(
//         pair(alt((char('*'), char('/'))), factor),
//         move || init,
//         |acc, (op, val): (char, i64)| {
//             if op == '*' {
//                 acc * val
//             } else {
//                 acc / val
//             }
//         },
//     )(i)
// }

// pub fn expr(i: &str) -> IResult<&str, Expression> {
//     let (i, init) = term(i)?;

//     fold_many0(
//         pair(alt((char('+'), char('-'))), term),
//         move || init,
//         |acc, (op, val): (char, i64)| {
//             if op == '+' {
//                 acc + val
//             } else {
//                 acc - val
//             }
//         },
//     )(i)
// }

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

    // #[test]
    // fn term_test() {
    //     assert_eq!(term(" 12 *2 /  3"), Ok(("", 8)));
    //     assert_eq!(term(" 2* 3  *2 *2 /  3"), Ok(("", 8)));
    //     assert_eq!(term(" 48 /  3/2"), Ok(("", 8)));
    // }

    // #[test]
    // fn expr_test() {
    //     assert_eq!(expr(" 1 +  2 "), Ok(("", 3)));
    //     assert_eq!(expr(" 12 + 6 - 4+  3"), Ok(("", 17)));
    //     assert_eq!(expr(" 1 + 2*3 + 4"), Ok(("", 11)));
    // }

    // #[test]
    // fn parens_test() {
    //     assert_eq!(expr(" (  2 )"), Ok(("", 2)));
    //     assert_eq!(expr(" 2* (  3 + 4 ) "), Ok(("", 14)));
    //     assert_eq!(expr("  2*2 / ( 5 - 1) + 3"), Ok(("", 4)));
    // }
}
