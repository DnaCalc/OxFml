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
pub enum SyntaxTriviaKind {
    Whitespace,
    Unknown,
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
pub struct SyntaxTrivia {
    pub kind: SyntaxTriviaKind,
    pub text: String,
    pub span: TextSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: TextSpan,
    pub leading_trivia: Vec<SyntaxTrivia>,
    pub trailing_trivia: Vec<SyntaxTrivia>,
}

impl Token {
    pub fn new(kind: TokenKind, text: impl Into<String>, span: TextSpan) -> Self {
        Self {
            kind,
            text: text.into(),
            span,
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }

    pub fn with_trivia(
        mut self,
        leading_trivia: Vec<SyntaxTrivia>,
        trailing_trivia: Vec<SyntaxTrivia>,
    ) -> Self {
        self.leading_trivia = leading_trivia;
        self.trailing_trivia = trailing_trivia;
        self
    }

    pub fn full_span(&self) -> TextSpan {
        let start = self
            .leading_trivia
            .first()
            .map(|trivia| trivia.span.start)
            .unwrap_or(self.span.start);
        let end = self
            .trailing_trivia
            .last()
            .map(|trivia| trivia.span.end())
            .unwrap_or(self.span.end());
        TextSpan::new(start, end.saturating_sub(start))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxDiagnostic {
    pub message: String,
    pub span: TextSpan,
}
