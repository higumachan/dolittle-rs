
use nom::{IResult, character::complete::{anychar, }, bytes::complete::{tag, }, combinator::{value, not, map}, sequence::{tuple, preceded}, multi::{many0, }, branch::alt, Err};
use std::ptr::eq;
use std::borrow::Borrow;
use core::symbol::{SymbolTable, SymbolId};
use std::cell::RefCell;
use nom::error::ErrorKind;
use std::rc::Rc;
use core::ast::ASTNode;
use nom::sequence::{terminated, separated_pair};
use unicode_num::ParseUnicodeExt;
use nom::combinator::iterator;
use nom::multi::many1;
use nom::bytes::complete::{take_until, take_till};
use nom::Err::Error;

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

fn not_symbol(input: &str) -> IResult<&str, ()> {
    value((), alt((
        value("", specials),
        nom_unicode::complete::space1,
    )))(input)
}

fn symbol(input: &str) -> IResult<&str, String> {
    map(tuple((
        map(preceded(
            not(alt((nom_unicode::complete::digit1, value("", not_symbol)))),
            anychar
        ), |c| c.to_string()),
        map(many0(preceded(
            not(not_symbol),
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
            num_value_static,
            method_call,
            form_without_method_call,
        )
    )(input)
}

fn form_without_method_call(input: &str) -> IResult<&str, ASTNode> {
    alt((
        num_value_static,
        decl,
    ))(input)
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

fn num(input: &str) -> IResult<&str, core::types::Value> {
    map(nom_unicode::complete::digit1, |x: &str| {
        core::types::Value::Num(x.parse_unicode().unwrap())
    })(input)
}

fn num_value_static(input: &str) -> IResult<&str, ASTNode> {
    map(num, |x| ASTNode::new_value_static(x))(input)
}

fn method_call(input: &str) -> IResult<&str, ASTNode> {
    map(tuple((
        last_symbol,
        form,
        preceded(
            nom_unicode::complete::space0,
            many0(terminated(form, nom_unicode::complete::space1)),
        ),
    )), |(method, object, args)| {
        ASTNode::new_method_call(
            method,
            object,
            args,
        )
    })(input)
}

#[derive(Clone)]
enum Token {
    SpecialToken(SpecialToken),
    ValueToken(core::types::Value),
    SymbolToken(String),
}

fn tokenize(input: &str) -> IResult<&str, Vec<Token>> {
    many1(terminated(alt((
        map(symbol, |x| Token::SymbolToken(x)),
        map(specials, |x| Token::SpecialToken(x)),
        map(num, |x| Token::ValueToken(x)),
    )), nom_unicode::complete::space0))(input)
}

fn last_symbol(input: &str) -> IResult<&str, String> {
    let tokens = tokenize(input);
    tokens.and_then(|x| {
        let mut a = x.1.clone();
        let mut t = None;
        while let Some(s) = a.pop() {
            match s {
                Token::SpecialToken(SpecialToken::EndOfTerm) => {
                }
                _ => {
                    t = Some(s);
                    break;
                }
            }
        }
        if let Some(Token::SymbolToken(sym)) = x.1.last() {
            Ok((&input[0..(input.len() - sym.len())], sym.clone()))
        } else {
            Err(Error((input, ErrorKind::Tag)))
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{specials, SpecialToken, symbol, decl, form, term, method_call, last_symbol};
    use nom::{
        IResult,
        Err,
        error::{ErrorKind},
        character,
    };
    use rstest::*;
    use core::ast::ASTNode;
    use core::types::Value;
    use nom::lib::std::collections::hash_map::Values;
    use unicode_num::ParseUnicodeExt;

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
    fn kameta_walk() {
        assert_eq!(
            term("かめた！１００　歩く。"),
            Ok(("", ASTNode::new_method_call(
                "歩く".to_string(),
                ASTNode::new_decl("かめた".to_string()),
                vec![
                    ASTNode::new_value_static(Value::Num(100.0)),
            ])))
        );
    }

    #[test]
    #[ignore]
    fn kameta_walk_turn_left90() {
        assert_eq!(term("かめた！１００　歩く　９０　右回り。"),
                   Ok(("", ASTNode::new_method_call("右回り".to_string(), ASTNode::new_method_call(
                       "歩く".to_string(),
                       ASTNode::new_decl("かめた".to_string()),
                       vec![
                           ASTNode::new_value_static(Value::Num(100.0)),
                       ]), vec![ASTNode::new_value_static(Value::Num(90.0))]))));
    }

    #[test]
    fn awesome_check() {
        let target = "タートル！作る";
        assert_eq!(decl(target), Ok(("作る", ASTNode::new_decl("タートル".to_string()))));

        assert_eq!("１００".parse_unicode(), Ok(100usize));

        assert_eq!(form("１００"), Ok(("",
                                   ASTNode::new_value_static(Value::Num(100.0)))));

        assert_eq!(last_symbol("タートル！作る"), Ok(("タートル！", "作る".to_string())));
    }

    #[rstest(input, expected,
        case("タートル！作る", Ok(("タートル！", "作る".to_string()))),
        case("タートル！作る　１００", Err(Err::Error(("タートル！作る　１００", ErrorKind::Tag)))),
        case("かめた！１００　歩く　９０　右回り", Ok(("かめた！１００　歩く　９０　", "右回り".to_string()))),
        case("タートル！作る。", Ok(("タートル！", "作る".to_string()))),
    )]
    fn parse_last_symbol(input: &str, expected: IResult<&str, String>) {
        assert_eq!(last_symbol(input), expected);
    }
}
