use crate::syntax::token::{TextSpan, Token, TokenKind};

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        let start = index;

        let token = match ch {
            '=' => simple(TokenKind::Equals, ch, start),
            '<' => match chars.get(index + 1).copied() {
                Some('=') => {
                    index += 1;
                    Token::new(TokenKind::LessEqual, "<=", TextSpan::new(start, 2))
                }
                Some('>') => {
                    index += 1;
                    Token::new(TokenKind::NotEqual, "<>", TextSpan::new(start, 2))
                }
                _ => simple(TokenKind::Less, ch, start),
            },
            '>' => match chars.get(index + 1).copied() {
                Some('=') => {
                    index += 1;
                    Token::new(TokenKind::GreaterEqual, ">=", TextSpan::new(start, 2))
                }
                _ => simple(TokenKind::Greater, ch, start),
            },
            '(' => simple(TokenKind::LParen, ch, start),
            ')' => simple(TokenKind::RParen, ch, start),
            ',' => simple(TokenKind::Comma, ch, start),
            ':' => simple(TokenKind::Colon, ch, start),
            '&' => simple(TokenKind::Ampersand, ch, start),
            '+' => simple(TokenKind::Plus, ch, start),
            '-' => simple(TokenKind::Minus, ch, start),
            '%' => simple(TokenKind::Percent, ch, start),
            '^' => simple(TokenKind::Caret, ch, start),
            '*' => simple(TokenKind::Star, ch, start),
            '/' => simple(TokenKind::Slash, ch, start),
            '.' if peek_is_ascii_digit(&chars, index + 1) => {
                index = consume_number_literal(&chars, index);
                Token::new(
                    TokenKind::Number,
                    chars[start..index].iter().collect::<String>(),
                    TextSpan::new(start, index - start),
                )
            }
            '.' => simple(TokenKind::Dot, ch, start),
            '@' => simple(TokenKind::At, ch, start),
            '#' => {
                if let Some(end) = consume_error_literal(&chars, start) {
                    index = end;
                    Token::new(
                        TokenKind::ErrorLiteral,
                        chars[start..index].iter().collect::<String>(),
                        TextSpan::new(start, index - start),
                    )
                } else {
                    simple(TokenKind::Hash, ch, start)
                }
            }
            '!' => simple(TokenKind::Bang, ch, start),
            '{' => simple(TokenKind::LBrace, ch, start),
            '}' => simple(TokenKind::RBrace, ch, start),
            ';' => simple(TokenKind::Semicolon, ch, start),
            '\'' => {
                index += 1;
                while index < chars.len() {
                    if chars[index] == '\'' {
                        if index + 1 < chars.len() && chars[index + 1] == '\'' {
                            index += 2;
                            continue;
                        }
                        index += 1;
                        break;
                    }
                    index += 1;
                }
                Token::new(
                    TokenKind::QuotedIdentifier,
                    chars[start..index].iter().collect::<String>(),
                    TextSpan::new(start, index - start),
                )
            }
            '[' => {
                consume_balanced_bracket_group(&chars, &mut index);
                while index < chars.len() && is_identifier_continue(chars[index]) {
                    index += 1;
                }
                Token::new(
                    TokenKind::BracketedQualifier,
                    chars[start..index].iter().collect::<String>(),
                    TextSpan::new(start, index - start),
                )
            }
            '"' => {
                index += 1;
                while index < chars.len() {
                    if chars[index] == '"' {
                        if index + 1 < chars.len() && chars[index + 1] == '"' {
                            index += 2;
                            continue;
                        }
                        index += 1;
                        break;
                    }
                    index += 1;
                }
                Token::new(
                    TokenKind::StringLiteral,
                    chars[start..index].iter().collect::<String>(),
                    TextSpan::new(start, index - start),
                )
            }
            c if c.is_ascii_whitespace() => {
                index += 1;
                while index < chars.len() && chars[index].is_ascii_whitespace() {
                    index += 1;
                }
                Token::new(
                    TokenKind::Whitespace,
                    chars[start..index].iter().collect::<String>(),
                    TextSpan::new(start, index - start),
                )
            }
            c if c.is_ascii_digit() || (c == '.' && peek_is_ascii_digit(&chars, index + 1)) => {
                index = consume_number_literal(&chars, index);
                Token::new(
                    TokenKind::Number,
                    chars[start..index].iter().collect::<String>(),
                    TextSpan::new(start, index - start),
                )
            }
            c if is_identifier_start(c) => {
                index += 1;
                while index < chars.len() {
                    if chars[index] == '.' && matches!(chars.get(index + 1), Some('@' | '*' | '['))
                    {
                        break;
                    }
                    if is_identifier_continue(chars[index]) {
                        index += 1;
                        continue;
                    }
                    if chars[index] == '[' {
                        consume_balanced_bracket_group(&chars, &mut index);
                        continue;
                    }
                    break;
                }
                Token::new(
                    TokenKind::Identifier,
                    chars[start..index].iter().collect::<String>(),
                    TextSpan::new(start, index - start),
                )
            }
            _ => simple(TokenKind::Unknown, ch, start),
        };

        if index < token.span.end() {
            index = token.span.end();
        }

        tokens.push(token);
    }

    tokens.push(Token::new(
        TokenKind::Eof,
        String::new(),
        TextSpan::new(input.len(), 0),
    ));
    tokens
}

fn simple(kind: TokenKind, ch: char, start: usize) -> Token {
    Token::new(kind, ch.to_string(), TextSpan::new(start, 1))
}

fn consume_error_literal(chars: &[char], start: usize) -> Option<usize> {
    const ERROR_LITERALS: &[&str] = &[
        "#GETTING_DATA",
        "#BLOCKED!",
        "#CONNECT!",
        "#FIELD!",
        "#SPILL!",
        "#VALUE!",
        "#DIV/0!",
        "#NAME?",
        "#CALC!",
        "#BUSY!",
        "#NULL!",
        "#REF!",
        "#NUM!",
        "#N/A",
    ];

    for literal in ERROR_LITERALS {
        let end = start + literal.chars().count();
        if end > chars.len() {
            continue;
        }
        let candidate = chars[start..end].iter().collect::<String>();
        if candidate.eq_ignore_ascii_case(literal) {
            return Some(end);
        }
    }

    None
}

fn peek_is_ascii_digit(chars: &[char], index: usize) -> bool {
    chars.get(index).is_some_and(|ch| ch.is_ascii_digit())
}

fn consume_number_literal(chars: &[char], mut index: usize) -> usize {
    let mut saw_digits = false;

    while index < chars.len() && chars[index].is_ascii_digit() {
        index += 1;
        saw_digits = true;
    }

    if index < chars.len() && chars[index] == '.' {
        index += 1;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
            saw_digits = true;
        }
    }

    if saw_digits && index < chars.len() && matches!(chars[index], 'E' | 'e') {
        let exponent_start = index;
        let mut probe = index + 1;
        if probe < chars.len() && matches!(chars[probe], '+' | '-') {
            probe += 1;
        }
        if peek_is_ascii_digit(chars, probe) {
            index = probe + 1;
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
            }
        } else {
            index = exponent_start;
        }
    }

    index
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '$')
}

fn consume_balanced_bracket_group(chars: &[char], index: &mut usize) {
    if *index >= chars.len() || chars[*index] != '[' {
        return;
    }

    let mut depth = 0usize;
    while *index < chars.len() {
        if chars[*index] == '\'' && is_structured_reference_escape(chars, *index) {
            *index += 2;
            continue;
        }
        match chars[*index] {
            '[' => {
                depth += 1;
            }
            ']' => {
                depth = depth.saturating_sub(1);
                *index += 1;
                if depth == 0 {
                    break;
                }
                continue;
            }
            _ => {}
        }
        *index += 1;
    }
}

fn is_structured_reference_escape(chars: &[char], index: usize) -> bool {
    matches!(chars.get(index + 1), Some('#' | '[' | ']' | '@' | '\''))
}
