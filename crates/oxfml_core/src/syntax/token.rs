#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSpan {
    pub start: usize,
    pub len: usize,
}

impl TextSpan {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn end(self) -> usize {
        self.start + self.len
    }

    pub fn covering(start: TextSpan, end: TextSpan) -> Self {
        Self::new(start.start, end.end().saturating_sub(start.start))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Equals,
    Number,
    Identifier,
    QuotedIdentifier,
    BracketedQualifier,
    StringLiteral,
    LParen,
    RParen,
    Comma,
    Colon,
    Plus,
    Minus,
    At,
    Hash,
    Bang,
    Whitespace,
    Unknown,
    Eof,
}

impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: TextSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxDiagnostic {
    pub message: String,
    pub span: TextSpan,
}
