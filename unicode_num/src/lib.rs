use std::str::FromStr;
use num::Num;

trait ParseUnicodeExt {
    fn parse_unicode<F: FromStr + Num>(&self) -> Result<F, F::Err>;
}

impl ParseUnicodeExt for str {
    fn parse_unicode<F: FromStr + Num>(&self) -> Result<F, <F as FromStr>::Err> {
        FromStr::from_str(to_ascii_digit(self).as_str())
    }
}


fn to_ascii_digit(input: &str) -> String {
    input.chars().map(|c| {
        match c {
            '０' => '0',
            '１' => '1',
            '２' => '2',
            '３' => '3',
            '４' => '4',
            '５' => '5',
            '６' => '6',
            '７' => '7',
            '８' => '8',
            '９' => '9',
            _ => c,
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use crate::{to_ascii_digit, ParseUnicodeExt};
    use std::num::{ParseIntError, ParseFloatError};
    use rstest::*;

    #[test]
    fn test_to_ascii_digit() {
        assert_eq!(to_ascii_digit("０１２３４５６７８９abc"), "0123456789abc".to_string())
    }

    #[rstest(input, expected,
        case("１２３", Ok(123usize)),
        case("123", Ok(123usize)),
    )]
    fn test_parse_unicode_usize(input: &str, expected: Result<usize, ParseIntError>) {
        assert_eq!(input.parse_unicode(), expected);
    }

    #[rstest(input, expected,
    case("１.２３", Ok(1.23f64)),
    case("1.23", Ok(1.23f64)),
    )]
    fn test_parse_unicode_f64(input: &str, expected: Result<f64, ParseFloatError>) {
        assert_eq!(input.parse_unicode(), expected);
    }
}
