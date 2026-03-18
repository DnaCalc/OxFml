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
            '(' => simple(TokenKind::LParen, ch, start),
            ')' => simple(TokenKind::RParen, ch, start),
            ',' => simple(TokenKind::Comma, ch, start),
            ':' => simple(TokenKind::Colon, ch, start),
            '+' => simple(TokenKind::Plus, ch, start),
            '-' => simple(TokenKind::Minus, ch, start),
            '@' => simple(TokenKind::At, ch, start),
            '#' => simple(TokenKind::Hash, ch, start),
            '!' => simple(TokenKind::Bang, ch, start),
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
                Token {
                    kind: TokenKind::QuotedIdentifier,
                    text: chars[start..index].iter().collect(),
                    span: TextSpan::new(start, index - start),
                }
            }
            '[' => {
                index += 1;
                while index < chars.len() && chars[index] != ']' {
                    index += 1;
                }
                if index < chars.len() {
                    index += 1;
                }
                while index < chars.len() && is_identifier_continue(chars[index]) {
                    index += 1;
                }
                Token {
                    kind: TokenKind::BracketedQualifier,
                    text: chars[start..index].iter().collect(),
                    span: TextSpan::new(start, index - start),
                }
            }
            '"' => {
                index += 1;
                while index < chars.len() && chars[index] != '"' {
                    index += 1;
                }
                if index < chars.len() {
                    index += 1;
                }
                Token {
                    kind: TokenKind::StringLiteral,
                    text: chars[start..index].iter().collect(),
                    span: TextSpan::new(start, index - start),
                }
            }
            c if c.is_ascii_whitespace() => {
                index += 1;
                while index < chars.len() && chars[index].is_ascii_whitespace() {
                    index += 1;
                }
                Token {
                    kind: TokenKind::Whitespace,
                    text: chars[start..index].iter().collect(),
                    span: TextSpan::new(start, index - start),
                }
            }
            c if c.is_ascii_digit() => {
                index += 1;
                while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.')
                {
                    index += 1;
                }
                Token {
                    kind: TokenKind::Number,
                    text: chars[start..index].iter().collect(),
                    span: TextSpan::new(start, index - start),
                }
            }
            c if is_identifier_start(c) => {
                index += 1;
                while index < chars.len() && is_identifier_continue(chars[index]) {
                    index += 1;
                }
                Token {
                    kind: TokenKind::Identifier,
                    text: chars[start..index].iter().collect(),
                    span: TextSpan::new(start, index - start),
                }
            }
            _ => simple(TokenKind::Unknown, ch, start),
        };

        if token.span.len == 1 && index == start {
            index += 1;
        }

        tokens.push(token);
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        text: String::new(),
        span: TextSpan::new(input.len(), 0),
    });
    tokens
}

fn simple(kind: TokenKind, ch: char, start: usize) -> Token {
    Token {
        kind,
        text: ch.to_string(),
        span: TextSpan::new(start, 1),
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '$')
}
