
use nom::{IResult, character::complete::{anychar, }, bytes::complete::{tag, }, combinator::{value, not, map}, sequence::{tuple, preceded}, multi::{many0, }, branch::alt, Err};
use std::ptr::eq;
use std::borrow::Borrow;
use core::symbol::{SymbolTable, SymbolId};
use std::cell::RefCell;
use nom::error::ErrorKind;
use std::rc::Rc;
use core::ast::ASTNode;
use nom::sequence::{terminated, separated_pair, delimited};
use unicode_num::ParseUnicodeExt;
use nom::combinator::{iterator, complete, opt};
use nom::multi::many1;
use nom::bytes::complete::{take_until, take_till};
use nom::Err::Error;
use core::ast::ASTNode::MethodCall;
use nom::character::complete::{line_ending};

pub fn parse_program_code(input: &str) -> IResult<&str, Vec<ASTNode>> {
    complete(many0(terminated(term, alt(
        (
            nom_unicode::complete::space0,
            line_ending,
        )
    ))))(input)
}

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
    Pipe,
    EndOfTerm,
    Comma,
    Colon,
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

fn open_parentheses(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::OpenParentheses, alt((tag("("), tag("（"))))(input)
}

fn close_parentheses(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::CloseParentheses, alt((tag(")"), tag(")"))))(input)
}

fn open_angles(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::OpenAngles, alt(
        (tag("「"), tag("["),)
    ))(input)
}

fn close_angles(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::CloseAngles, alt(
        (tag("」"), tag("]"),)
    ))(input)
}

fn pipe(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::Pipe, alt(
        (tag("｜"), tag("|"),)
    ))(input)
}

fn comma(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::Comma, alt(
        (tag("、"), tag(","),)
    ))(input)
}

fn colon(input: &str) -> IResult<&str, SpecialToken> {
    value(SpecialToken::Colon, alt(
        (tag("："), tag(":"))
    ))(input)
}

fn specials(input: &str) -> IResult<&str, SpecialToken> {
    let plus = value(SpecialToken::Plus, alt((tag("+"), tag("＋"))));
    let minus = value(SpecialToken::Minus, (tag("-")));

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
        comma,
        pipe,
        colon,
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
            method_call,
            num_static_value,
            block,
            decl,
        )
    )(input)
}

fn symbol_or_parentheses_form(input: &str) -> IResult<&str, ASTNode> {
    alt(
        (
            map(symbol, |x| ASTNode::new_decl(&None, x.as_str())),
            delimited(open_parentheses, form, close_parentheses),
        )
    )(input)
}

fn symbol_or_member(input: &str) -> IResult<&str, (Option<ASTNode>, String)> {
    alt((
        map(tuple((terminated(symbol_or_parentheses_form, colon), symbol)), |(x, y)| (Some(x), y)),
        map(symbol, |x| (None, x)),
    ))(input)
}

fn assign(input: &str) -> IResult<&str, ASTNode> {
    map(separated_pair(
        symbol_or_member,
        equal,
        form,
    ),
        |((object_ast, sym), value)| ASTNode::new_assign(
            &object_ast, sym.as_str(), &value)
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
        symbol_or_member,
        |(object_ast, symbol_id)| {
            ASTNode::new_decl(&object_ast, &symbol_id)
        })(input)
}

fn num(input: &str) -> IResult<&str, core::types::Value> {
    map(nom_unicode::complete::digit1, |x: &str| {
        core::types::Value::Num(x.parse_unicode().unwrap())
    })(input)
}

fn num_static_value(input: &str) -> IResult<&str, ASTNode> {
    map(num, |x| ASTNode::new_static_value(&x))(input)
}


fn single_value(input: &str) -> IResult<&str, ASTNode> {
    alt(
        (
            num_static_value,
            decl,
            delimited(open_parentheses, form, close_parentheses)
        )
    )(input)
}


fn single_value_without_decl(input: &str) -> IResult<&str, ASTNode> {
    alt(
        (
            num_static_value,
            delimited(open_parentheses, form, close_parentheses)
        )
    )(input)
}

fn method_call(input: &str) -> IResult<&str, ASTNode> {
    let parse_method_calls = tuple((
        preceded(
            nom_unicode::complete::space0,
            many0(terminated(
                single_value_without_decl,
                nom_unicode::complete::space1
            )),
        ),
        symbol,
    )
    );
    map(tuple((
        terminated(
        single_value,
        exclamation,
        ),
        many1(
            parse_method_calls
        ),
    )), |(object, method_calls)| {
        let mut method_calls: Vec<(Vec<ASTNode>, String)> = method_calls;
        method_calls.into_iter().fold(object, |node, x| {
            ASTNode::new_method_call(&x.1, &node, &x.0)
        })
    })(input)
}

fn dummy_args_list(input: &str) -> IResult<&str, Vec<String>> {
    delimited(pipe,
    map(opt(map(tuple((
        symbol,
        many0(preceded(tuple((
            nom_unicode::complete::space0,
            comma,
            nom_unicode::complete::space0,
        )), symbol))
    )), |(first, mut remain)| {
        remain.insert(0, first);
        remain
    })), |x| x.unwrap_or(vec![])),
pipe)(input)
}

fn block(input: &str) -> IResult<&str, ASTNode> {
    map(delimited(
        open_angles,
        tuple((
            dummy_args_list,
            preceded(
                nom_unicode::complete::space0,
                many0(
                    terminated(term, nom_unicode::complete::space0))
            ),
        )),
        close_angles,
    ), |(dummy_args, terms)| {
        ASTNode::new_block_define(
            &dummy_args.iter().map(|x| x.as_str()).collect(),
            &terms
        )
    })(input)
}


#[cfg(test)]
mod tests {
    use crate::{specials, SpecialToken, symbol, decl, form, term, method_call, parse_program_code, block, dummy_args_list, symbol_or_member, assign};
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
    use std::rc::Rc;

    #[test]
    fn test_parse_program_code() {
        let result = parse_program_code(r#"かめた＝タートル！作る。
かめた！１００　歩く。
"#);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1.len(), 2);
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
        case("かめた", Ok(("", ASTNode::new_decl(&None, "かめた")))),
        case("かめた！", Ok(("！", ASTNode::new_decl(&None, "かめた")))),
        case("かめた:歩幅", Ok(("", ASTNode::new_decl(&Some(ASTNode::new_decl(&None, "かめた")), "歩幅")))),
    )]
    fn parse_decl(input: &str, expected: IResult<&str, ASTNode>) {
        assert_eq!(decl(input), expected);
    }

    #[rstest(input, expected,
        case("タートル！作る", Ok(
        (
            "",
            ASTNode::new_method_call("作る", &ASTNode::new_decl(&None, "タートル"), &vec![])
        ))),
        case("かめた！(歩幅)　歩く", Ok((
            "",
            ASTNode::new_method_call(
                "歩く",
                &ASTNode::new_decl(&None, "かめた"),
                &vec![ASTNode::new_decl(&None, "歩幅")]
            )
        )))
    )]
    fn parse_method_call(input: &str, expected: IResult<&str, ASTNode>) {
        assert_eq!(method_call(input), expected);
    }

    #[rstest(input, expected,
        case("｜歩幅｜", Ok(("", vec!["歩幅".to_string()]))),
        case("|歩幅|", Ok(("", vec!["歩幅".to_string()]))),
        case("|歩幅, 角度|", Ok(("", vec!["歩幅".to_string(), "角度".to_string()]))),
    )]
    fn parse_dummy_args_list(input: &str, expected: IResult<&str, Vec<String>>) {
        assert_eq!(dummy_args_list(input), expected);
    }

    #[rstest(input, expected,
        case("「|歩幅| かめた！(歩幅)　歩く。」", Ok(
        (
            "",
            ASTNode::new_block_define(&vec!["歩幅"], &vec![ASTNode::new_method_call(
                "歩く",
                &ASTNode::new_decl(&None, "かめた"),
                &vec![ASTNode::new_decl(&None, "歩幅")]
            )])
        ))),
        case("「|歩幅|かめた！(歩幅)　歩く。」", Ok(
        (
            "",
            ASTNode::new_block_define(&vec!["歩幅"], &vec![ASTNode::new_method_call(
                "歩く",
                &ASTNode::new_decl(&None, "かめた"),
                &vec![ASTNode::new_decl(&None, "歩幅")]
            )])
        ))),
        case("「||かめた！100　歩く。」", Ok(
        (
            "",
            ASTNode::new_block_define(&vec![], &vec![ASTNode::new_method_call(
                "歩く",
                &ASTNode::new_decl(&None, "かめた"),
                &vec![ASTNode::new_static_value(&Value::Num(100.0))]
            )])
        ))),
    )]
    fn parse_block(input: &str, expected: IResult<&str, ASTNode>) {
        assert_eq!(block(input), expected);
    }

    #[rstest(input, expected,
        case("X", Ok(("", (None, "X".to_string())))),
        case("かめた：X", Ok(("", (Some(ASTNode::new_decl(&None, "かめた")), "X".to_string())))),
        case("かめた：歩く２", Ok(("", (Some(ASTNode::new_decl(&None, "かめた")), "歩く２".to_string())))),
    )]
    fn parse_symbol_or_member(input: &str, expected: IResult<&str, (Option<ASTNode>, String)>) {
        assert_eq!(symbol_or_member(input), expected);
    }

    #[test]
    fn kameta_create() {
        assert_eq!(
            term("かめた＝タートル！作る。"),
            Ok(("", ASTNode::new_assign(
                &None,
                "かめた",
                &ASTNode::new_method_call(
                    "作る",
                    &ASTNode::new_decl(&None, "タートル"),
                    &vec![]
                ))
            ))
        );
    }

    #[test]
    fn kameta_walk() {
        assert_eq!(
            term("かめた！１００　歩く。"),
            Ok(("", ASTNode::new_method_call(
                "歩く",
                &ASTNode::new_decl(
                    &None,
                    "かめた"),
                &vec![
                    ASTNode::new_static_value(&Value::Num(100.0)),
            ])))
        );
    }

    #[test]
    fn kameta_walk_turn_left90() {
        assert_eq!(term("かめた！１００　歩く　９０　右回り。"),
                   Ok(("", ASTNode::new_method_call("右回り", &ASTNode::new_method_call(
                       "歩く",
                       &ASTNode::new_decl(&None, "かめた"),
                       &vec![
                           ASTNode::new_static_value(&Value::Num(100.0)),
                       ]), &vec![ASTNode::new_static_value(&Value::Num(90.0))]))));
    }

    #[rstest(input, expected,
        case("かめた：歩幅＝１００", Ok(("", ASTNode::new_assign(
            &Some(ASTNode::new_decl(&None, "かめた")),
            "歩幅",
            &ASTNode::new_static_value(&Value::Num(100.0)),
        )))),
    )]
    fn parse_assign(input: &str, expected: IResult<&str, ASTNode>) {
        assert_eq!(assign(input), expected);
    }

    #[test]
    fn method_assign_test() {
        let target = "かめた：歩く２＝「｜N｜　かめた！(N)　歩く。　かめた！(N)　歩く。」。";
        assert_eq!(term(target), Ok(("", ASTNode::new_assign(
            &Some(ASTNode::new_decl(&None, "かめた")), "歩く２", &ASTNode::new_block_define(
                &vec!["N"], &vec![
                    ASTNode::new_method_call("歩く",
                                             &ASTNode::new_decl(&None, "かめた"),
                                             &vec![ASTNode::new_decl(&None, "N")]),
                    ASTNode::new_method_call("歩く",
                                             &ASTNode::new_decl(&None, "かめた"),
                                             &vec![ASTNode::new_decl(&None, "N")]),
                ]
            )
        ))));
    }

    #[test]
    fn awesome_check() {
        let target = "タートル！作る";
        assert_eq!(decl(target), Ok(("！作る", ASTNode::new_decl(&None, "タートル"))));

        assert_eq!("１００".parse_unicode(), Ok(100usize));

        assert_eq!(form("１００"), Ok(("",
                                   ASTNode::new_static_value(&Value::Num(100.0)))));
    }
}
