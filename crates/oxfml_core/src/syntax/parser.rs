use crate::source::FormulaSourceRecord;
use crate::syntax::green::{GreenChild, GreenNode, GreenTreeRoot, SyntaxKind};
use crate::syntax::lexer::lex;
use crate::syntax::token::{
    SyntaxDiagnostic, SyntaxTrivia, SyntaxTriviaKind, TextSpan, Token, TokenKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRequest {
    pub source: FormulaSourceRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub green_tree: GreenTreeRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalParseResult {
    pub green_tree: GreenTreeRoot,
    pub reused_green_tree: bool,
}

pub fn parse_formula(request: ParseRequest) -> ParseResult {
    let full_tokens = lex(&request.source.entered_formula_text);
    let mut parser = Parser::new(full_tokens.clone());
    let root = attach_trivia_to_green_tree(parser.parse_formula_root(), &full_tokens);
    ParseResult {
        green_tree: GreenTreeRoot::from_parts(root, full_tokens, parser.diagnostics),
    }
}

pub fn parse_formula_incremental(
    request: ParseRequest,
    previous_green_tree: Option<&GreenTreeRoot>,
) -> IncrementalParseResult {
    if let Some(previous_green_tree) = previous_green_tree {
        let previous_text = previous_green_tree
            .full_fidelity_tokens
            .iter()
            .filter(|token| token.kind != TokenKind::Eof)
            .map(|token| token.text.as_str())
            .collect::<String>();
        if previous_text == request.source.entered_formula_text {
            return IncrementalParseResult {
                green_tree: previous_green_tree.clone(),
                reused_green_tree: true,
            };
        }
    }

    let parse = parse_formula(request);
    IncrementalParseResult {
        green_tree: parse.green_tree,
        reused_green_tree: false,
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
        self.parse_comparison(allow_union_comma)
    }

    fn parse_comparison(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut left = self.parse_concat(allow_union_comma);
        while self.at(TokenKind::Equals)
            || self.at(TokenKind::NotEqual)
            || self.at(TokenKind::Less)
            || self.at(TokenKind::LessEqual)
            || self.at(TokenKind::Greater)
            || self.at(TokenKind::GreaterEqual)
        {
            let op = self.bump();
            let right = self.parse_concat(allow_union_comma);
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

    fn parse_concat(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut left = self.parse_additive(allow_union_comma);
        while self.at(TokenKind::Ampersand) {
            let op = self.bump();
            let right = self.parse_additive(allow_union_comma);
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

    fn parse_additive(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut left = self.parse_multiplicative(allow_union_comma);
        while self.at(TokenKind::Plus) || self.at(TokenKind::Minus) {
            let op = self.bump();
            let right = self.parse_multiplicative(allow_union_comma);
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

    fn parse_multiplicative(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut left = self.parse_power(allow_union_comma);
        while self.at(TokenKind::Star) || self.at(TokenKind::Slash) {
            let op = self.bump();
            let right = self.parse_power(allow_union_comma);
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

    fn parse_power(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut left = self.parse_percent(allow_union_comma);
        while self.at(TokenKind::Caret) {
            let op = self.bump();
            let right = self.parse_percent(allow_union_comma);
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

    fn parse_percent(&mut self, allow_union_comma: bool) -> GreenNode {
        let mut node = self.parse_prefix(allow_union_comma);
        while self.at(TokenKind::Percent) {
            let percent = self.bump();
            node = GreenNode::new(
                SyntaxKind::PostfixExpr,
                vec![GreenChild::Node(Box::new(node)), GreenChild::Token(percent)],
            );
        }
        node
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
        let mut left = self.parse_range(allow_union_comma);
        loop {
            let spaces = self.take_whitespace_tokens();
            if spaces.is_empty() || !self.starts_reference_expr() {
                return left;
            }

            let right = self.parse_range(allow_union_comma);
            let mut children = vec![GreenChild::Node(Box::new(left))];
            children.extend(spaces.into_iter().map(GreenChild::Token));
            children.push(GreenChild::Node(Box::new(right)));
            left = GreenNode::new(SyntaxKind::IntersectionExpr, children);

            if !allow_union_comma {
                return left;
            }
        }
    }

    fn parse_range(&mut self, allow_union_comma: bool) -> GreenNode {
        self.skip_whitespace();
        if self.at(TokenKind::At) {
            let at = self.bump();
            let expr = self.parse_range(allow_union_comma);
            return GreenNode::new(
                SyntaxKind::PrefixExpr,
                vec![GreenChild::Token(at), GreenChild::Node(Box::new(expr))],
            );
        }

        let mut left = self.parse_postfix();
        loop {
            if self.at(TokenKind::Colon) {
                let colon = self.bump();
                self.skip_whitespace();
                let right = self.parse_postfix();
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

    fn parse_prefix(&mut self, allow_union_comma: bool) -> GreenNode {
        self.skip_whitespace();
        if self.at(TokenKind::Plus) || self.at(TokenKind::Minus) {
            let op = self.bump();
            let expr = self.parse_prefix(allow_union_comma);
            GreenNode::new(
                SyntaxKind::PrefixExpr,
                vec![GreenChild::Token(op), GreenChild::Node(Box::new(expr))],
            )
        } else {
            self.parse_union(allow_union_comma)
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
            TokenKind::Identifier | TokenKind::QuotedIdentifier | TokenKind::BracketedQualifier => {
                let ident = self.bump();
                if self.at(TokenKind::Bang) {
                    let bang = self.bump();
                    self.skip_whitespace();
                    let target = self.parse_primary();
                    GreenNode::new(
                        SyntaxKind::QualifiedReferenceExpr,
                        vec![
                            GreenChild::Token(ident),
                            GreenChild::Token(bang),
                            GreenChild::Node(Box::new(target)),
                        ],
                    )
                } else if ident.kind == TokenKind::Identifier && self.at(TokenKind::LParen) {
                    self.parse_call_expr(ident)
                } else if ident.kind == TokenKind::QuotedIdentifier {
                    GreenNode::new(
                        SyntaxKind::QuotedIdentifierExpr,
                        vec![GreenChild::Token(ident)],
                    )
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
            TokenKind::LBrace => self.parse_array_literal(),
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
                if self.at(TokenKind::Comma) {
                    args_children.push(GreenChild::Node(Box::new(self.omitted_argument_node())));
                } else {
                    args_children.push(GreenChild::Node(Box::new(self.parse_expression(false))));
                }
                self.skip_whitespace();
                if self.at(TokenKind::Comma) {
                    args_children.push(GreenChild::Token(self.bump()));
                    self.skip_whitespace();
                    if self.at(TokenKind::RParen) {
                        args_children
                            .push(GreenChild::Node(Box::new(self.omitted_argument_node())));
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        let close = self.expect(TokenKind::RParen, "expected ')'");
        args_children.push(GreenChild::Token(close));
        GreenNode::new(SyntaxKind::ArgumentList, args_children)
    }

    fn parse_array_literal(&mut self) -> GreenNode {
        let open = self.expect(TokenKind::LBrace, "expected '{'");
        let mut children = vec![GreenChild::Token(open)];
        self.skip_whitespace();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            // Inside array literals, comma and semicolon are element/row separators, not union
            // operators, so parse each element with union-comma disabled.
            children.push(GreenChild::Node(Box::new(self.parse_expression(false))));
            self.skip_whitespace();
            if self.at(TokenKind::Comma) || self.at(TokenKind::Semicolon) {
                children.push(GreenChild::Token(self.bump()));
                self.skip_whitespace();
            } else {
                break;
            }
        }
        let close = self.expect(TokenKind::RBrace, "expected '}'");
        children.push(GreenChild::Token(close));
        GreenNode::new(SyntaxKind::ArrayLiteralExpr, children)
    }

    fn omitted_argument_node(&self) -> GreenNode {
        GreenNode::new(SyntaxKind::OmittedArgExpr, Vec::new())
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
            TokenKind::Identifier
                | TokenKind::QuotedIdentifier
                | TokenKind::BracketedQualifier
                | TokenKind::Number
                | TokenKind::At
                | TokenKind::LParen
                | TokenKind::LBrace
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
            Token::new(kind, String::new(), token.span)
        }
    }
}

fn attach_trivia_to_green_tree(root: GreenNode, full_tokens: &[Token]) -> GreenNode {
    let significant_whitespace_spans = collect_significant_whitespace_spans(&root);
    let mut attached_tokens: Vec<Token> = Vec::new();
    let mut pending_trivia = Vec::new();
    let mut previous_nontrivia_index: Option<usize> = None;

    for token in full_tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Eof)
    {
        if token.kind.is_trivia() {
            if significant_whitespace_spans.contains(&token.span) {
                continue;
            }
            pending_trivia.push(SyntaxTrivia {
                kind: syntax_trivia_kind(token.kind),
                text: token.text.clone(),
                span: token.span,
            });
            continue;
        }

        if let Some(previous_nontrivia_index) = previous_nontrivia_index {
            attached_tokens[previous_nontrivia_index]
                .trailing_trivia
                .extend(pending_trivia.iter().cloned());
        }

        let leading_trivia = std::mem::take(&mut pending_trivia);
        attached_tokens.push(token.clone().with_trivia(leading_trivia, Vec::new()));
        previous_nontrivia_index = Some(attached_tokens.len() - 1);
    }

    if let Some(previous_nontrivia_index) = previous_nontrivia_index {
        attached_tokens[previous_nontrivia_index]
            .trailing_trivia
            .extend(pending_trivia);
    }

    let mut next_attached_index = 0usize;
    rehydrate_green_node_tokens(root, &attached_tokens, &mut next_attached_index)
}

fn collect_significant_whitespace_spans(node: &GreenNode) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    collect_significant_whitespace_spans_recursive(node, &mut spans);
    spans
}

fn collect_significant_whitespace_spans_recursive(node: &GreenNode, spans: &mut Vec<TextSpan>) {
    for child in &node.children {
        match child {
            GreenChild::Node(child_node) => {
                collect_significant_whitespace_spans_recursive(child_node, spans);
            }
            GreenChild::Token(token) if token.kind.is_trivia() => spans.push(token.span),
            GreenChild::Token(_) => {}
        }
    }
}

fn rehydrate_green_node_tokens(
    node: GreenNode,
    attached_tokens: &[Token],
    next_attached_index: &mut usize,
) -> GreenNode {
    let children = node
        .children
        .into_iter()
        .map(|child| match child {
            GreenChild::Node(child_node) => GreenChild::Node(Box::new(
                rehydrate_green_node_tokens(*child_node, attached_tokens, next_attached_index),
            )),
            GreenChild::Token(token) => {
                let updated_token = if token.kind.is_trivia() || token.kind == TokenKind::Eof {
                    token
                } else if let Some(attached_token) = attached_tokens.get(*next_attached_index) {
                    if attached_token.kind == token.kind
                        && attached_token.text == token.text
                        && attached_token.span == token.span
                    {
                        *next_attached_index += 1;
                        attached_token.clone()
                    } else {
                        token
                    }
                } else {
                    token
                };
                GreenChild::Token(updated_token)
            }
        })
        .collect();

    GreenNode {
        kind: node.kind,
        span: node.span,
        children,
    }
}

fn syntax_trivia_kind(token_kind: TokenKind) -> SyntaxTriviaKind {
    match token_kind {
        TokenKind::Whitespace => SyntaxTriviaKind::Whitespace,
        _ => SyntaxTriviaKind::Unknown,
    }
}
