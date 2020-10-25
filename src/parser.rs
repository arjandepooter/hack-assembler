use crate::types::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, line_ending, multispace0, not_line_ending, space0},
    combinator::{map, map_res, opt, value, verify},
    error::{convert_error, make_error, ErrorKind, ParseError, VerboseError},
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    Err, FindToken, IResult,
};

fn eof<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, (), E> {
    if input.is_empty() {
        Ok((input, ()))
    } else {
        Err(Err::Error(make_error(input, ErrorKind::Eof)))
    }
}

fn eol_or_eof<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, (), E> {
    value((), alt((eof, value((), line_ending))))(input)
}

fn comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, Instruction, E> {
    value(
        Instruction::Noop,
        tuple((multispace0, tag("//"), not_line_ending)),
    )(input)
}

fn label<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, Instruction, E> {
    map(delimited(char('('), label_string, char(')')), |s: &str| {
        Instruction::Label(s.to_string())
    })(input)
}

fn label_string<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    take_while1(|c: char| c.is_alphanumeric() || ":$._".find_token(c))(input)
}

fn a_label<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, AValue, E> {
    map(label_string, |s: &str| AValue::Label(s.to_string()))(input)
}

fn a_value<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, AValue, E> {
    map(
        verify(map_res(digit1, |s: &str| s.parse::<u16>()), |v| {
            v < &2u16.pow(15)
        }),
        AValue::Value,
    )(input)
}

fn a_instruction<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, Instruction, E> {
    map(
        preceded(char('@'), alt((a_value, a_label))),
        Instruction::AInstruction,
    )(input)
}

fn destination<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, u8, E> {
    map(
        terminated(
            tuple((opt(char('A')), opt(char('M')), opt(char('D')))),
            char('='),
        ),
        |(a, m, d)| {
            a.map(|_| 4).unwrap_or_default()
                + d.map(|_| 2).unwrap_or_default()
                + m.map(|_| 1).unwrap_or_default()
        },
    )(input)
}

fn operation<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, u8, E> {
    let operations = [
        ("D+1", 0b0011111),
        ("A+1", 0b0110111),
        ("M+1", 0b1110111),
        ("D-1", 0b0001110),
        ("A-1", 0b0110010),
        ("M-1", 0b1110010),
        ("D+A", 0b0000010),
        ("D+M", 0b1000010),
        ("D-A", 0b0010011),
        ("D-M", 0b1010011),
        ("A-D", 0b0000111),
        ("M-D", 0b1000111),
        ("D&A", 0b0000000),
        ("D&M", 0b1000000),
        ("D|A", 0b0010101),
        ("D|M", 0b1010101),
        ("0", 0b0101010),
        ("1", 0b0111111),
        ("-1", 0b0111010),
        ("D", 0b0001100),
        ("A", 0b0110000),
        ("M", 0b1110000),
        ("!D", 0b0001101),
        ("!A", 0b0110001),
        ("!M", 0b1110001),
        ("-D", 0b0001111),
        ("-A", 0b0110011),
        ("-M", 0b1110011),
    ];

    operations.iter().fold(
        Err(Err::Error(make_error(input, ErrorKind::Char))),
        |acc, (op, val)| match acc {
            Ok(_) => acc,
            Err(_) => value(*val, tag(*op))(input),
        },
    )
}

fn jump<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, u8, E> {
    map(
        preceded(
            char(';'),
            alt((
                tag("JGT"),
                tag("JEQ"),
                tag("JGE"),
                tag("JLT"),
                tag("JNE"),
                tag("JLE"),
                tag("JMP"),
            )),
        ),
        |s: &str| match s {
            "JGT" => 1,
            "JEQ" => 2,
            "JGE" => 3,
            "JLT" => 4,
            "JNE" => 5,
            "JLE" => 6,
            "JMP" => 7,
            _ => 0,
        },
    )(input)
}

fn c_instruction<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, Instruction, E> {
    map(
        tuple((opt(destination), operation, opt(jump))),
        |(destination, instruction, jump)| Instruction::CInstruction {
            instruction,
            destination: destination.unwrap_or(0),
            jump: jump.unwrap_or(0),
        },
    )(input)
}

fn instruction<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, Instruction, E> {
    terminated(
        preceded(
            multispace0,
            alt((label, comment, a_instruction, c_instruction)),
        ),
        tuple((space0, opt(comment), eol_or_eof)),
    )(input)
}
fn root<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Vec<Instruction>, E> {
    terminated(many0(instruction), eof)(input).map(|(i, instructions)| {
        (
            i,
            instructions
                .into_iter()
                .filter(|instruction| match instruction {
                    Instruction::Noop => false,
                    _ => true,
                })
                .collect(),
        )
    })
}

pub fn parse_file(contents: &str) -> Result<Vec<Instruction>, String> {
    match root::<VerboseError<&str>>(contents) {
        Ok((_, instructions)) => Ok(instructions),
        Err(Err::Error(err)) => Err(convert_error(contents, err)),
        _ => Err("Unrecoverable error".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type SimpleErr = (&'static str, ErrorKind);

    #[test]
    fn comment_with_newline() {
        assert_eq!(
            comment::<SimpleErr>("// abddef\r\n"),
            Ok(("\r\n", Instruction::Noop))
        );
    }

    #[test]
    fn comment_with_leading_whitespace() {
        assert_eq!(
            comment::<SimpleErr>("    //   dsadas"),
            Ok(("", Instruction::Noop))
        );
    }

    #[test]
    fn simple_label() {
        assert_eq!(
            label::<SimpleErr>("(nice)"),
            Ok(("", Instruction::Label("nice".to_string())))
        )
    }

    #[test]
    fn label_with_surrounding_whitespaces() {
        assert_eq!(
            label::<SimpleErr>("(noice) "),
            Ok((" ", Instruction::Label("noice".to_string())))
        )
    }

    #[test]
    fn instruction_with_trailing_comment() {
        assert_eq!(
            instruction::<SimpleErr>("(somelabel)          // test "),
            Ok(("", Instruction::Label("somelabel".to_string())))
        )
    }

    #[test]
    fn instruction_with_extra_spaces() {
        assert_eq!(
            instruction::<SimpleErr>("       (some_label)    \r\n"),
            Ok(("", Instruction::Label("some_label".to_string())))
        )
    }

    #[test]
    fn a_instruction_with_value() {
        assert_eq!(
            a_instruction::<SimpleErr>("@1234"),
            Ok(("", Instruction::AInstruction(AValue::Value(1234))))
        )
    }

    #[test]
    fn a_instruction_with_label() {
        assert_eq!(
            a_instruction::<SimpleErr>("@OUTPUT_LABEL"),
            Ok((
                "",
                Instruction::AInstruction(AValue::Label("OUTPUT_LABEL".to_string()))
            ))
        )
    }

    #[test]
    fn a_instruction_with_value_out_of_range_u16() {
        assert_eq!(
            a_instruction::<SimpleErr>("@654654"),
            Ok((
                "",
                Instruction::AInstruction(AValue::Label("654654".to_string()))
            ))
        )
    }

    #[test]
    fn a_instruction_with_value_out_of_range_u15() {
        assert_eq!(
            a_instruction::<SimpleErr>("@35000"),
            Ok((
                "",
                Instruction::AInstruction(AValue::Label("35000".to_string()))
            ))
        )
    }

    #[test]
    fn a_instruction_with_trailing_non_digits() {
        assert_eq!(
            instruction::<SimpleErr>("@1337abc"),
            Ok((
                "",
                Instruction::AInstruction(AValue::Label("1337abc".to_string()))
            ))
        )
    }

    #[test]
    fn destination_() {
        assert_eq!(destination::<SimpleErr>("A="), Ok(("", 4)));
        assert_eq!(destination::<SimpleErr>("M="), Ok(("", 1)));
        assert_eq!(destination::<SimpleErr>("D="), Ok(("", 2)));
        assert_eq!(destination::<SimpleErr>("AMD="), Ok(("", 7)));
        assert_eq!(destination::<SimpleErr>("AD="), Ok(("", 6)));
        assert_eq!(
            destination::<SimpleErr>("DA="),
            Err(Err::Error(("A=", ErrorKind::Char)))
        )
    }

    #[test]
    fn operation_() {
        let parser = operation::<SimpleErr>;
        assert_eq!(parser("A+1"), Ok(("", 0b0110111)));
        assert_eq!(parser("D   "), Ok(("   ", 0b0001100)));
    }

    #[test]
    fn c_instruction_() {
        let parser = c_instruction::<SimpleErr>;

        assert_eq!(
            parser("A=D+1;JEQ"),
            Ok((
                "",
                Instruction::CInstruction {
                    instruction: 0b0011111,
                    destination: 4,
                    jump: 2
                }
            ))
        );
        assert_eq!(
            parser("D=D+A"),
            Ok((
                "",
                Instruction::CInstruction {
                    instruction: 0b10,
                    destination: 2,
                    jump: 0
                }
            ))
        );
    }
}
