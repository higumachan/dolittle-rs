
use nom::{
    IResult,
    character::complete::{anychar,},
    bytes::complete::{tag,},
    combinator::{value, not, map},
    sequence::{tuple, preceded},
    multi::{many0,},
    branch::alt,
};
use std::ptr::eq;
use std::borrow::Borrow;

#[derive(Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
enum SpecialToken {
    Plus,
    Minus,
    Exclamation,
    Equal,
    OpenParentheses,
    CloseParentheses,
    OpenAngles,
    CloseAngles,
}


fn specials(input: &str) -> IResult<&str, SpecialToken> {
    let plus = value(SpecialToken::Plus, alt((tag("+"), tag("＋"))));
    let minus = value(SpecialToken::Minus, alt((tag("-"), tag("ー"))));
    let exclamation = value(SpecialToken::Exclamation, alt((tag("!"), tag("！"))));
    let equal = value(SpecialToken::Equal, alt((tag("="), tag("＝"))));
    let open_parentheses = value(SpecialToken::OpenParentheses, alt((tag("("), tag("（"))));
    let close_parentheses = value(SpecialToken::CloseParentheses, alt((tag(")"), tag(")"))));
    let open_angles = value(SpecialToken::OpenAngles ,alt((tag("["), tag("｢"))));
    let close_angles = value(SpecialToken::CloseAngles, alt((tag("]"), tag("｣"))));

    alt((
        plus,
        minus,
        exclamation,
        equal,
        open_parentheses,
        close_parentheses,
        open_angles,
        close_angles,
    ))(input)
}

fn symbol(input: &str) -> IResult<&str, String> {
    map(tuple((
        map(preceded(
            not(alt((nom_unicode::complete::digit1, value("", specials)) )),
            anychar
        ), |c| c.to_string()),
        map(many0(preceded(
            not(specials),
            anychar
        )), |x| { x.iter().collect::<String>() }),
    )), |(s1, s2)| {
        (s1 + s2.as_str())
    })(input)
}

#[cfg(test)]
mod tests {
    use crate::{specials, SpecialToken, symbol};
    use nom::{
        IResult,
        Err,
        error::{ErrorKind},
        character,
    };
    use rstest::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[rstest(input, expected,
        case("+", Ok(("", SpecialToken::Plus))),
        case("＋", Ok(("", SpecialToken::Plus))),
        case("1", Err(Err::Error(("1", ErrorKind::Tag)))),
        case("=", Ok(("", SpecialToken::Equal))),
    )]
    fn parse_specials(input: &str, expected: IResult<&str, SpecialToken>) {
        assert_eq!(specials(input), expected);
    }

    #[rstest(input, expected,
        case("なでこ", Ok(("", "なでこ".to_string())))
    )]
    fn parse_symbol(input: &str, expected: IResult<&str, String>) {
        assert_eq!(symbol(input), expected);
    }
}
