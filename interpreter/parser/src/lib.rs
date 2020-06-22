
use nom::{IResult, character::complete::{anychar, }, bytes::complete::{tag, }, combinator::{value, not, map}, sequence::{tuple, preceded}, multi::{many0, }, branch::alt, Err};
use std::ptr::eq;
use std::borrow::Borrow;
use core::symbol::{SymbolTable, SymbolId};
use std::cell::RefCell;
use nom::error::ErrorKind;
use std::rc::Rc;
use core::ast::ASTNode;
use nom::sequence::{terminated, separated_pair};

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
    EndOfTerm,
}

fn exclamation(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::Exclamation, alt((tag("!"), tag("！"))))(input)
}

fn equal(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::Equal, alt((tag("="), tag("＝"))))(input)
}

fn end_of_term(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::EndOfTerm, tag("。"))(input)
}

fn specials(input: &str) -> IResult<&str, SpecialToken> {
    let plus = value(SpecialToken::Plus, alt((tag("+"), tag("＋"))));
    let minus = value(SpecialToken::Minus, (tag("-")));
    let open_parentheses = value(SpecialToken::OpenParentheses, alt((tag("("), tag("（"))));
    let close_parentheses = value(SpecialToken::CloseParentheses, alt((tag(")"), tag(")"))));
    let open_angles = value(SpecialToken::OpenAngles ,alt((tag("["), tag("｢"))));
    let close_angles = value(SpecialToken::CloseAngles, alt((tag("]"), tag("｣"))));

    alt((
        plus,
        minus,
        exclamation,
        equal,
        end_of_term,
        open_parentheses,
        close_parentheses,
        open_angles,
        close_angles,
    ))(input)
}

fn symbol(input: &str) -> IResult<&str, String> {
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
        s
    })(input)
}

fn form(input: &str) -> IResult<&str, ASTNode> {
    alt(
        (
            method_call,
            form_without_method_call,
        )
    )(input)
}

fn form_without_method_call(input: &str) -> IResult<&str, ASTNode> {
    decl(input)
}

fn assign(input: &str) -> IResult<&str, ASTNode> {
    map(separated_pair(
        symbol,
        equal,
        form,
    ),
        |(sym, value)| ASTNode::new_assign(sym, Box::new(value))
    )(input)
}

fn term(input: &str) -> IResult<&str, ASTNode> {
    terminated(
        alt(
            (
                assign,
                form,
            )
        ),
        end_of_term,
    )(input)
}

fn decl(input: &str) -> IResult<&str, ASTNode> {
    map(
        tuple((symbol, exclamation)),
        |(symbol_id, _)| ASTNode::new_decl(symbol_id))(input)
}

fn method_call(input: &str) -> IResult<&str, ASTNode> {
    map(tuple((
        form_without_method_call,
        many0(form),
        symbol,
    )), |(object, args, method)| {
        ASTNode::new_method_call(
            method,
            object,
            args,
        )
    })(input)
}

#[cfg(test)]
mod tests {
    use crate::{specials, SpecialToken, symbol, decl, form, term, method_call};
    use nom::{
        IResult,
        Err,
        error::{ErrorKind},
        character,
    };
    use rstest::*;
    use core::ast::ASTNode;

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

    #[rstest(input, expected,
        case("かめた！", Ok(("", ASTNode::new_decl("かめた".to_string())))),
    )]
    fn parse_decl(input: &str, expected: IResult<&str, ASTNode>) {
        assert_eq!(decl(input), expected);
    }

    #[rstest(input, expected,
        case("タートル！作る", Ok(
        (
            "",
            ASTNode::new_method_call("作る".to_string(), ASTNode::new_decl("タートル".to_string()), vec![])
        )))
    )]
    fn parse_method_call(input: &str, expected: IResult<&str, ASTNode>) {
        assert_eq!(method_call(input), expected);
    }

    #[test]
    fn kameta_create() {
        assert_eq!(
            term("かめた＝タートル！作る。"),
            Ok(("", ASTNode::new_assign(
                "かめた".to_string(),
                Box::new(ASTNode::new_method_call(
                    "作る".to_string(),
                    ASTNode::new_decl("タートル".to_string()),
                    vec![]
                ))
            )))
        );
    }

    #[test]
    fn awesome_check() {
        let target = "タートル！作る";
        assert_eq!(decl(target), Ok(("作る", ASTNode::new_decl("タートル".to_string()))));
    }
}
