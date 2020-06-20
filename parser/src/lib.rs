
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
use core::ast::{MethodCall, Decl};
use core::symbol::{SymbolTable, SymbolId};
use std::cell::RefCell;

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

fn exclamation(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::Exclamation, alt((tag("!"), tag("！"))))(input)
}

fn specials(input: &str) -> IResult<&str, SpecialToken> {
    let plus = value(SpecialToken::Plus, alt((tag("+"), tag("＋"))));
    let minus = value(SpecialToken::Minus, alt((tag("-"), tag("ー"))));
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

struct DolittleParser {
    symbol_table: RefCell<SymbolTable>,
}

impl DolittleParser {
    fn symbol(&self, input: &str) -> IResult<&str, SymbolId> {
        map(tuple((
            map(preceded(
                not(alt((nom_unicode::complete::digit1, value("", specials)))),
                anychar
            ), |c| c.to_string()),
            map(many0(preceded(
                not(specials),
                anychar
            )), |x| {
                x.iter().collect::<String>()
            }),
        )), |(s1, s2)| {
            let s = (s1 + s2.as_str());
            self.symbol_table.borrow_mut().insert_user_symbol_if_no_exist(s.as_str())
        })(input)
    }

    fn decl(&self, input: &str) -> IResult<&str, Decl> {
        map(tuple((self.symbol, exclamation)), |(symbol_id, _)| Decl { taget: symbol_id })
    }

    fn method_call(input: &str) -> IResult<&str, MethodCall> {}
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
        case("なでこ", Ok(("", "なでこ".to_string()))),
        case("なでこ1", Ok(("", "なでこ1".to_string()))),
        case("なでこ１", Ok(("", "なでこ１".to_string()))),
        case("１なでこ", Err(Err::Error(("１なでこ", ErrorKind::Not)))),
        case("！なでこ", Err(Err::Error(("！なでこ", ErrorKind::Not)))),
    )]
    fn parse_symbol(input: &str, expected: IResult<&str, String>) {
        assert_eq!(symbol(input), expected);
    }
}
