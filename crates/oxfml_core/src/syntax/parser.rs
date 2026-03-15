use crate::source::FormulaSourceRecord;
use crate::syntax::green::{GreenChild, GreenNode, GreenTreeRoot, SyntaxKind};
use crate::syntax::lexer::lex;
use crate::syntax::token::{SyntaxDiagnostic, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRequest {
    pub source: FormulaSourceRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub green_tree: GreenTreeRoot,
}

pub fn parse_formula(request: ParseRequest) -> ParseResult {
    let full_tokens = lex(&request.source.entered_formula_text);
    let mut parser = Parser::new(full_tokens.clone());
    let root = parser.parse_formula_root();
    ParseResult {
        green_tree: GreenTreeRoot::from_parts(root, full_tokens, parser.diagnostics),
    }
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
    diagnostics: Vec<SyntaxDiagnostic>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            index: 0,
            diagnostics: Vec::new(),
        }
    }

    fn parse_formula_root(&mut self) -> GreenNode {
        let mut children = Vec::new();
        if self.at(TokenKind::Equals) {
            children.push(GreenChild::Token(self.bump()));
        }
        self.skip_whitespace();
        children.push(GreenChild::Node(Box::new(self.parse_expression(true))));
        self.skip_whitespace();
        if !self.at(TokenKind::Eof) {
            let token = self.current().clone();
            self.diagnostics.push(SyntaxDiagnostic {
                message: format!("unexpected trailing token {:?}", token.kind),
                span: token.span,
            });
        }
        children.push(GreenChild::Token(
            self.expect(TokenKind::Eof, "expected end of formula"),
        ));
        GreenNode::new(SyntaxKind::FormulaRoot, children)
    }

    fn parse_expression(&mut self, allow_union_comma: bool) -> GreenNode {
        self.parse_additive(allow_union_comma)
    }

    fn parse_additive(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut left = self.parse_union(allow_union_comma);
        while self.at(TokenKind::Plus) || self.at(TokenKind::Minus) {
            let op = self.bump();
            let right = self.parse_union(allow_union_comma);
            left = GreenNode::new(
                SyntaxKind::BinaryExpr,
                vec![
                    GreenChild::Node(Box::new(left)),
                    GreenChild::Token(op),
                    GreenChild::Node(Box::new(right)),
                ],
            );
        }
        left
    }

    fn parse_union(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut left = self.parse_intersection(allow_union_comma);
        loop {
            self.skip_whitespace();
            if allow_union_comma && self.at(TokenKind::Comma) {
                let comma = self.bump();
                self.skip_whitespace();
                let right = self.parse_intersection(allow_union_comma);
                left = GreenNode::new(
                    SyntaxKind::UnionExpr,
                    vec![
                        GreenChild::Node(Box::new(left)),
                        GreenChild::Token(comma),
                        GreenChild::Node(Box::new(right)),
                    ],
                );
            } else {
                return left;
            }
        }
    }

    fn parse_intersection(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut left = self.parse_range();
        loop {
            let spaces = self.take_whitespace_tokens();
            if spaces.is_empty() || !self.starts_reference_expr() {
                return left;
            }

            let right = self.parse_range();
            let mut children = vec![GreenChild::Node(Box::new(left))];
            children.extend(spaces.into_iter().map(GreenChild::Token));
            children.push(GreenChild::Node(Box::new(right)));
            left = GreenNode::new(SyntaxKind::IntersectionExpr, children);

            if !allow_union_comma {
                return left;
            }
        }
    }

    fn parse_range(&mut self) -> GreenNode {
        let mut left = self.parse_prefix();
        loop {
            if self.at(TokenKind::Colon) {
                let colon = self.bump();
                self.skip_whitespace();
                let right = self.parse_prefix();
                left = GreenNode::new(
                    SyntaxKind::RangeExpr,
                    vec![
                        GreenChild::Node(Box::new(left)),
                        GreenChild::Token(colon),
                        GreenChild::Node(Box::new(right)),
                    ],
                );
            } else {
                return left;
            }
        }
    }

    fn parse_postfix(&mut self) -> GreenNode {
        let mut node = self.parse_primary();
        loop {
            if self.at(TokenKind::Hash) {
                let hash = self.bump();
                node = GreenNode::new(
                    SyntaxKind::PostfixExpr,
                    vec![GreenChild::Node(Box::new(node)), GreenChild::Token(hash)],
                );
            } else if self.at(TokenKind::LParen) {
                let args = self.parse_argument_list();
                node = GreenNode::new(
                    SyntaxKind::InvokeExpr,
                    vec![
                        GreenChild::Node(Box::new(node)),
                        GreenChild::Node(Box::new(args)),
                    ],
                );
            } else {
                break;
            }
        }
        node
    }

    fn parse_prefix(&mut self) -> GreenNode {
        self.skip_whitespace();
        if self.at(TokenKind::At) {
            let at = self.bump();
            let expr = self.parse_prefix();
            GreenNode::new(
                SyntaxKind::PrefixExpr,
                vec![GreenChild::Token(at), GreenChild::Node(Box::new(expr))],
            )
        } else {
            self.parse_postfix()
        }
    }

    fn parse_primary(&mut self) -> GreenNode {
        self.skip_whitespace();
        match self.current().kind {
            TokenKind::Number => GreenNode::new(
                SyntaxKind::NumberLiteralExpr,
                vec![GreenChild::Token(self.bump())],
            ),
            TokenKind::StringLiteral => GreenNode::new(
                SyntaxKind::StringLiteralExpr,
                vec![GreenChild::Token(self.bump())],
            ),
            TokenKind::Identifier => {
                let ident = self.bump();
                if self.at(TokenKind::LParen) {
                    self.parse_call_expr(ident)
                } else {
                    GreenNode::new(SyntaxKind::IdentifierExpr, vec![GreenChild::Token(ident)])
                }
            }
            TokenKind::LParen => {
                let open = self.bump();
                let expr = self.parse_expression(true);
                let close = self.expect(TokenKind::RParen, "expected ')'");
                GreenNode::new(
                    SyntaxKind::GroupingExpr,
                    vec![
                        GreenChild::Token(open),
                        GreenChild::Node(Box::new(expr)),
                        GreenChild::Token(close),
                    ],
                )
            }
            _ => {
                let token = self.current().clone();
                self.diagnostics.push(SyntaxDiagnostic {
                    message: format!("unexpected token {:?}", token.kind),
                    span: token.span,
                });
                GreenNode::new(
                    SyntaxKind::MissingExpr,
                    vec![GreenChild::Token(self.bump())],
                )
            }
        }
    }

    fn parse_call_expr(&mut self, ident: Token) -> GreenNode {
        let args = self.parse_argument_list();
        GreenNode::new(
            SyntaxKind::CallExpr,
            vec![GreenChild::Token(ident), GreenChild::Node(Box::new(args))],
        )
    }

    fn parse_argument_list(&mut self) -> GreenNode {
        let open = self.expect(TokenKind::LParen, "expected '('");
        let mut args_children = vec![GreenChild::Token(open)];
        self.skip_whitespace();
        if !self.at(TokenKind::RParen) {
            while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                args_children.push(GreenChild::Node(Box::new(self.parse_expression(false))));
                self.skip_whitespace();
                if self.at(TokenKind::Comma) {
                    args_children.push(GreenChild::Token(self.bump()));
                    self.skip_whitespace();
                } else {
                    break;
                }
            }
        }
        let close = self.expect(TokenKind::RParen, "expected ')'");
        args_children.push(GreenChild::Token(close));
        GreenNode::new(SyntaxKind::ArgumentList, args_children)
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    fn skip_whitespace(&mut self) {
        while self.at(TokenKind::Whitespace) {
            self.bump();
        }
    }

    fn take_whitespace_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.at(TokenKind::Whitespace) {
            tokens.push(self.bump());
        }
        tokens
    }

    fn starts_reference_expr(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Identifier | TokenKind::At | TokenKind::LParen
        )
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        if self.index < self.tokens.len().saturating_sub(1) {
            self.index += 1;
        }
        token
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Token {
        if self.at(kind) {
            self.bump()
        } else {
            let token = self.current().clone();
            self.diagnostics.push(SyntaxDiagnostic {
                message: message.to_string(),
                span: token.span,
            });
            Token {
                kind,
                text: String::new(),
                span: token.span,
            }
        }
    }
}
