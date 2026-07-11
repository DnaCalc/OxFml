use crate::binding::{ReferenceBindProfile, ReferenceSelectorKind, ReferenceSelectorSyntax};
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReferenceSelectorSyntaxProfile {
    pub selectors: Vec<ReferenceSelectorSyntax>,
}

impl ReferenceSelectorSyntaxProfile {
    pub fn from_selectors(selectors: impl IntoIterator<Item = ReferenceSelectorSyntax>) -> Self {
        Self {
            selectors: selectors.into_iter().collect(),
        }
    }

    pub fn from_bind_profile(profile: Option<&dyn ReferenceBindProfile>) -> Self {
        profile
            .map(|profile| Self::from_selectors(profile.selector_syntax()))
            .unwrap_or_default()
    }

    pub fn collection_family_for(&self, token_text: &str) -> Option<&str> {
        let normalized = normalize_reference_selector_token(token_text);
        self.selectors
            .iter()
            .find(|selector| {
                selector.selector_kind == ReferenceSelectorKind::Collection
                    && normalize_reference_selector_token(&selector.token_text) == normalized
            })
            .map(|selector| selector.selector_family.as_str())
    }

    pub fn structural_selector_family_for(&self, token_text: &str) -> Option<&str> {
        let normalized = normalize_reference_selector_token(token_text);
        self.selectors
            .iter()
            .find(|selector| {
                selector.selector_kind == ReferenceSelectorKind::StructuralSelector
                    && normalize_reference_selector_token(&selector.token_text) == normalized
            })
            .map(|selector| selector.selector_family.as_str())
    }

    pub fn selector_family_for(&self, token_text: &str) -> Option<&str> {
        self.collection_family_for(token_text)
            .or_else(|| self.structural_selector_family_for(token_text))
    }
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
    parse_formula_with_reference_selector_syntax(
        request,
        &ReferenceSelectorSyntaxProfile::default(),
    )
}

pub fn parse_formula_with_reference_selector_syntax(
    request: ParseRequest,
    reference_selector_syntax: &ReferenceSelectorSyntaxProfile,
) -> ParseResult {
    let full_tokens = lex(&request.source.entered_formula_text);
    if let Some(root) = worksheet_cell_entry_literal_root(&request.source, &full_tokens) {
        return ParseResult {
            green_tree: GreenTreeRoot::from_parts(root, full_tokens, Vec::new()),
        };
    }

    let mut parser = Parser::new(full_tokens.clone(), reference_selector_syntax.clone());
    let root = attach_trivia_to_green_tree(parser.parse_formula_root(), &full_tokens);
    ParseResult {
        green_tree: GreenTreeRoot::from_parts(root, full_tokens, parser.diagnostics),
    }
}

pub fn parse_formula_incremental(
    request: ParseRequest,
    previous_green_tree: Option<&GreenTreeRoot>,
) -> IncrementalParseResult {
    parse_formula_incremental_with_reference_selector_syntax(
        request,
        previous_green_tree,
        &ReferenceSelectorSyntaxProfile::default(),
    )
}

pub fn parse_formula_incremental_with_reference_selector_syntax(
    request: ParseRequest,
    previous_green_tree: Option<&GreenTreeRoot>,
    reference_selector_syntax: &ReferenceSelectorSyntaxProfile,
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

    let parse = parse_formula_with_reference_selector_syntax(request, reference_selector_syntax);
    IncrementalParseResult {
        green_tree: parse.green_tree,
        reused_green_tree: false,
    }
}

fn worksheet_cell_entry_literal_root(
    source: &FormulaSourceRecord,
    full_tokens: &[Token],
) -> Option<GreenNode> {
    if source.formula_channel_kind != crate::source::FormulaChannelKind::WorksheetA1 {
        return None;
    }

    let raw = source.entered_formula_text.as_str();
    if raw.is_empty() || raw.starts_with('=') {
        return None;
    }

    let span = TextSpan::new(0, raw.chars().count());
    let expr = if let Some(forced_text) = raw.strip_prefix('\'') {
        literal_expr_node(
            SyntaxKind::StringLiteralExpr,
            TokenKind::StringLiteral,
            encode_string_literal_text(forced_text),
            span,
        )
    } else if raw.eq_ignore_ascii_case("TRUE") || raw.eq_ignore_ascii_case("FALSE") {
        literal_expr_node(
            SyntaxKind::IdentifierExpr,
            TokenKind::Identifier,
            raw.to_string(),
            span,
        )
    } else if raw.parse::<f64>().is_ok_and(f64::is_finite) {
        literal_expr_node(
            SyntaxKind::NumberLiteralExpr,
            TokenKind::Number,
            raw.to_string(),
            span,
        )
    } else if is_single_quoted_string_literal_token(full_tokens) {
        literal_expr_node(
            SyntaxKind::StringLiteralExpr,
            TokenKind::StringLiteral,
            raw.to_string(),
            span,
        )
    } else {
        literal_expr_node(
            SyntaxKind::StringLiteralExpr,
            TokenKind::StringLiteral,
            encode_string_literal_text(raw),
            span,
        )
    };

    let eof = full_tokens.last().cloned().unwrap_or_else(|| {
        Token::new(
            TokenKind::Eof,
            String::new(),
            TextSpan::new(raw.chars().count(), 0),
        )
    });

    Some(GreenNode::new(
        SyntaxKind::FormulaRoot,
        vec![GreenChild::Node(Box::new(expr)), GreenChild::Token(eof)],
    ))
}

fn literal_expr_node(
    syntax_kind: SyntaxKind,
    token_kind: TokenKind,
    text: String,
    span: TextSpan,
) -> GreenNode {
    GreenNode::new(
        syntax_kind,
        vec![GreenChild::Token(Token::new(token_kind, text, span))],
    )
}

fn is_single_quoted_string_literal_token(full_tokens: &[Token]) -> bool {
    let mut non_eof_tokens = full_tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Eof && !token.kind.is_trivia());
    matches!(
        (non_eof_tokens.next(), non_eof_tokens.next()),
        (Some(token), None) if token.kind == TokenKind::StringLiteral
    )
}

fn encode_string_literal_text(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
    diagnostics: Vec<SyntaxDiagnostic>,
    reference_selector_syntax: ReferenceSelectorSyntaxProfile,
}

impl Parser {
    fn new(tokens: Vec<Token>, reference_selector_syntax: ReferenceSelectorSyntaxProfile) -> Self {
        Self {
            tokens,
            index: 0,
            diagnostics: Vec::new(),
            reference_selector_syntax,
        }
    }

    fn parse_formula_root(&mut self) -> GreenNode {
        let mut children = Vec::new();
        self.skip_whitespace();
        if self.at(TokenKind::Equals) {
            children.push(GreenChild::Token(self.bump()));
        }
        self.skip_whitespace();
        children.push(GreenChild::Node(Box::new(self.parse_expression(true))));
        self.skip_whitespace();
        while !self.at(TokenKind::Eof) {
            let token = self.current().clone();
            self.diagnostics.push(SyntaxDiagnostic {
                message: format!("unexpected trailing token {:?}", token.kind),
                span: token.span,
            });
            children.push(GreenChild::Token(self.bump()));
            self.skip_whitespace();
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

    // `allow_union_comma` is forwarded to nested sub-parses; only the
    // intersection/union loop in the caller branches on it.
    #[allow(clippy::only_used_in_recursion)]
    fn parse_range(&mut self, allow_union_comma: bool) -> GreenNode {
        self.skip_whitespace();
        if self.at(TokenKind::At) {
            let at = self.bump();
            self.skip_whitespace();
            if self.at(TokenKind::Identifier) || self.at(TokenKind::Star) {
                let member = self.bump_reference_selector_token();
                if let Some((selector, dot, tail)) =
                    self.split_reference_selector_tail_token(&member)
                {
                    let base = GreenNode::new(
                        SyntaxKind::HostReferenceCollectionExpr,
                        vec![GreenChild::Token(at), GreenChild::Token(selector)],
                    );
                    return GreenNode::new(
                        SyntaxKind::HostMemberReferenceExpr,
                        vec![
                            GreenChild::Node(Box::new(base)),
                            GreenChild::Token(dot),
                            GreenChild::Token(tail),
                        ],
                    );
                }
                if self
                    .reference_selector_syntax
                    .selector_family_for(&member.text)
                    .is_some()
                {
                    let base = GreenNode::new(
                        SyntaxKind::HostReferenceCollectionExpr,
                        vec![GreenChild::Token(at), GreenChild::Token(member)],
                    );
                    return self.parse_host_member_reference_tails(base);
                }
                let primary =
                    GreenNode::new(SyntaxKind::IdentifierExpr, vec![GreenChild::Token(member)]);
                let operand = self.parse_range_from_primary(primary);
                return GreenNode::new(
                    SyntaxKind::PrefixExpr,
                    vec![GreenChild::Token(at), GreenChild::Node(Box::new(operand))],
                );
            }
            let expr = self.parse_range(allow_union_comma);
            return GreenNode::new(
                SyntaxKind::PrefixExpr,
                vec![GreenChild::Token(at), GreenChild::Node(Box::new(expr))],
            );
        }
        if self.at(TokenKind::Caret) {
            return self.parse_ancestor_anchor_expr();
        }
        if self.at(TokenKind::Dot) {
            let dot = self.bump();
            self.skip_whitespace();
            let at = self.at(TokenKind::At).then(|| self.bump());
            self.skip_whitespace();
            if self.at(TokenKind::Identifier) || self.at(TokenKind::Star) {
                let member = self.bump_reference_selector_token();
                if let Some((selector, split_dot, tail)) =
                    self.split_reference_selector_tail_token(&member)
                {
                    let mut children = vec![GreenChild::Token(dot)];
                    if let Some(at) = at {
                        children.push(GreenChild::Token(at));
                    }
                    children.push(GreenChild::Token(selector));
                    let base = GreenNode::new(SyntaxKind::HostReferenceCollectionExpr, children);
                    return GreenNode::new(
                        SyntaxKind::HostMemberReferenceExpr,
                        vec![
                            GreenChild::Node(Box::new(base)),
                            GreenChild::Token(split_dot),
                            GreenChild::Token(tail),
                        ],
                    );
                }
                if self
                    .reference_selector_syntax
                    .selector_family_for(&member.text)
                    .is_some()
                {
                    let mut children = vec![GreenChild::Token(dot)];
                    if let Some(at) = at {
                        children.push(GreenChild::Token(at));
                    }
                    children.push(GreenChild::Token(member));
                    return GreenNode::new(SyntaxKind::HostReferenceCollectionExpr, children);
                }
            }
            self.diagnostics.push(SyntaxDiagnostic {
                message: "expected host reference collection selector".to_string(),
                span: dot.span,
            });
            return GreenNode::new(SyntaxKind::MissingExpr, vec![GreenChild::Token(dot)]);
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
        let node = self.parse_primary();
        self.parse_postfix_from(node)
    }

    /// Continue a range expression (`:` chains, with each operand carrying its
    /// `#`/`(`/`.` postfixes) from an already-parsed primary node. Lets the `@`
    /// implicit-intersection prefix apply to a full reference expression
    /// (`@A1:A3`, `@A1#`, `@SEQUENCE(3)`) rather than only the leading atom.
    fn parse_range_from_primary(&mut self, primary: GreenNode) -> GreenNode {
        let mut left = self.parse_postfix_from(primary);
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

    fn parse_postfix_from(&mut self, mut node: GreenNode) -> GreenNode {
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
            } else if self.at(TokenKind::Dot) {
                let dot = self.bump();
                self.skip_whitespace();
                let at = self.at(TokenKind::At).then(|| self.bump());
                self.skip_whitespace();
                let member = if self.at(TokenKind::Identifier)
                    || self.at(TokenKind::Star)
                    || self.at(TokenKind::BracketedQualifier)
                {
                    self.bump_reference_selector_token()
                } else {
                    self.expect(TokenKind::Identifier, "expected host member selector")
                };
                if let Some(at_token) = at {
                    if let Some((selector, split_dot, tail)) =
                        self.split_reference_selector_tail_token(&member)
                    {
                        let first = GreenNode::new(
                            SyntaxKind::HostMemberReferenceExpr,
                            vec![
                                GreenChild::Node(Box::new(node)),
                                GreenChild::Token(dot),
                                GreenChild::Token(at_token),
                                GreenChild::Token(selector),
                            ],
                        );
                        node = GreenNode::new(
                            SyntaxKind::HostMemberReferenceExpr,
                            vec![
                                GreenChild::Node(Box::new(first)),
                                GreenChild::Token(split_dot),
                                GreenChild::Token(tail),
                            ],
                        );
                        continue;
                    }
                    let mut children = vec![
                        GreenChild::Node(Box::new(node)),
                        GreenChild::Token(dot),
                        GreenChild::Token(at_token),
                    ];
                    children.push(GreenChild::Token(member));
                    node = GreenNode::new(SyntaxKind::HostMemberReferenceExpr, children);
                    continue;
                }
                let mut children = vec![GreenChild::Node(Box::new(node)), GreenChild::Token(dot)];
                children.push(GreenChild::Token(member));
                node = GreenNode::new(SyntaxKind::HostMemberReferenceExpr, children);
            } else {
                break;
            }
        }
        node
    }

    fn parse_host_member_reference_tails(&mut self, mut node: GreenNode) -> GreenNode {
        loop {
            self.skip_whitespace();
            if !self.at(TokenKind::Dot) {
                return node;
            }
            let dot = self.bump();
            self.skip_whitespace();
            let at = self.at(TokenKind::At).then(|| self.bump());
            self.skip_whitespace();
            let member = if self.at(TokenKind::Identifier)
                || self.at(TokenKind::Star)
                || self.at(TokenKind::BracketedQualifier)
            {
                self.bump_reference_selector_token()
            } else {
                self.expect(TokenKind::Identifier, "expected host member selector")
            };
            if let Some(at_token) = at {
                if let Some((selector, split_dot, tail)) =
                    self.split_reference_selector_tail_token(&member)
                {
                    let first = GreenNode::new(
                        SyntaxKind::HostMemberReferenceExpr,
                        vec![
                            GreenChild::Node(Box::new(node)),
                            GreenChild::Token(dot),
                            GreenChild::Token(at_token),
                            GreenChild::Token(selector),
                        ],
                    );
                    node = GreenNode::new(
                        SyntaxKind::HostMemberReferenceExpr,
                        vec![
                            GreenChild::Node(Box::new(first)),
                            GreenChild::Token(split_dot),
                            GreenChild::Token(tail),
                        ],
                    );
                    continue;
                }
                let mut children = vec![
                    GreenChild::Node(Box::new(node)),
                    GreenChild::Token(dot),
                    GreenChild::Token(at_token),
                ];
                children.push(GreenChild::Token(member));
                node = GreenNode::new(SyntaxKind::HostMemberReferenceExpr, children);
                continue;
            }
            let mut children = vec![GreenChild::Node(Box::new(node)), GreenChild::Token(dot)];
            children.push(GreenChild::Token(member));
            node = GreenNode::new(SyntaxKind::HostMemberReferenceExpr, children);
        }
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

    fn parse_ancestor_anchor_expr(&mut self) -> GreenNode {
        let first = self.bump();
        let mut node = GreenNode::new(
            SyntaxKind::HostReferenceCollectionExpr,
            vec![GreenChild::Token(Token::new(
                TokenKind::Identifier,
                "SELF",
                first.span,
            ))],
        );
        while self.at(TokenKind::Caret) {
            let caret = self.bump();
            node = GreenNode::new(
                SyntaxKind::HostMemberReferenceExpr,
                vec![
                    GreenChild::Node(Box::new(node)),
                    GreenChild::Token(Token::new(TokenKind::Identifier, "PARENT", caret.span)),
                ],
            );
        }
        if self.at(TokenKind::Dot) {
            let dot = self.bump();
            self.skip_whitespace();
            let member = if self.at(TokenKind::Identifier) || self.at(TokenKind::Star) {
                self.bump_reference_selector_token()
            } else {
                self.expect(TokenKind::Identifier, "expected host member selector")
            };
            node = GreenNode::new(
                SyntaxKind::HostMemberReferenceExpr,
                vec![
                    GreenChild::Node(Box::new(node)),
                    GreenChild::Token(dot),
                    GreenChild::Token(member),
                ],
            );
        }
        node
    }

    fn split_reference_selector_tail_token(&self, token: &Token) -> Option<(Token, Token, Token)> {
        let (head, tail) = token.text.split_once('.')?;
        if tail.is_empty()
            || self
                .reference_selector_syntax
                .selector_family_for(head)
                .is_none()
        {
            return None;
        }
        let selector = Token::new(
            token.kind,
            head.to_string(),
            TextSpan::new(token.span.start, head.len()),
        );
        let dot = Token::new(
            TokenKind::Dot,
            ".",
            TextSpan::new(token.span.start + head.len(), 1),
        );
        let tail = Token::new(
            TokenKind::Identifier,
            tail.to_string(),
            TextSpan::new(token.span.start + head.len() + 1, tail.len()),
        );
        Some((selector, dot, tail))
    }

    fn bump_reference_selector_token(&mut self) -> Token {
        let first = self.bump();
        if first.kind != TokenKind::Star
            || !self.at(TokenKind::Star)
            || self
                .reference_selector_syntax
                .selector_family_for("**")
                .is_none()
        {
            return first;
        }
        let second = self.bump();
        Token::new(
            TokenKind::Identifier,
            "**",
            TextSpan::new(first.span.start, second.span.end() - first.span.start),
        )
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
            TokenKind::Identifier
            | TokenKind::ErrorLiteral
            | TokenKind::QuotedIdentifier
            | TokenKind::BracketedQualifier => {
                let ident = self.bump();
                if matches!(
                    ident.kind,
                    TokenKind::Identifier | TokenKind::QuotedIdentifier
                ) && self.at_sheet_span_tail()
                {
                    // W078 — 3D sheet span `Sheet1:Sheet3!<target>`. The bounded
                    // `Colon Identifier Bang` lookahead already confirmed the
                    // shape, so consume `: end_sheet !` here: this claims the
                    // `:` for the span production *before* the outer range loop
                    // can split on it (the collision the workset documents). The
                    // target is a single primary (`A1`); a trailing `:B2`
                    // (`Sheet1:Sheet3!A1:B2`) is left for the range loop, which
                    // then forms `RangeExpr(SheetSpan3D, B2)` — the exact
                    // parallel of the qualified-range `Sheet1!A1:B2` shape.
                    self.skip_whitespace();
                    let colon = self.bump(); // Colon
                    self.skip_whitespace();
                    let end_sheet = self.bump(); // Identifier | QuotedIdentifier
                    self.skip_whitespace();
                    let bang = self.bump(); // Bang
                    self.skip_whitespace();
                    let target = self.parse_primary();
                    GreenNode::new(
                        SyntaxKind::SheetSpan3DReferenceExpr,
                        vec![
                            GreenChild::Token(ident),
                            GreenChild::Token(colon),
                            GreenChild::Token(end_sheet),
                            GreenChild::Token(bang),
                            GreenChild::Node(Box::new(target)),
                        ],
                    )
                } else if self.at(TokenKind::Bang) {
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

    /// The kinds of the next `count` significant (non-whitespace) tokens
    /// starting at the current position, in order. Used for bounded,
    /// whitespace-skipping lookahead (W078 sheet-span detection). Stops at
    /// `Eof`; the returned vec may be shorter than `count` if the stream ends.
    fn peek_significant_kinds(&self, count: usize) -> Vec<TokenKind> {
        let mut kinds = Vec::with_capacity(count);
        let mut i = self.index;
        while kinds.len() < count && i < self.tokens.len() {
            let kind = self.tokens[i].kind;
            i += 1;
            if kind == TokenKind::Whitespace {
                continue;
            }
            if kind == TokenKind::Eof {
                break;
            }
            kinds.push(kind);
        }
        kinds
    }

    /// True when the position immediately *after* an already-bumped leading
    /// identifier begins a 3D sheet-span tail `: Identifier !` (whitespace
    /// skipped). The trailing `!` is mandatory: it is what distinguishes a
    /// sheet span from an ordinary `A1:A3` range or a `name:name` union member,
    /// which never carry it. Bounded to exactly three significant tokens.
    ///
    /// The end sheet is restricted to a *plain* `Identifier` (not
    /// `QuotedIdentifier`) on purpose: broadening it to admit a quoted end
    /// sheet collides with the tested whole-column qualified-range form
    /// `'Sheet'!A:'Sheet'!C`, whose inner target `A` would otherwise eat the
    /// `:'Sheet'!` tail as a bogus span. A quoted *start* sheet is safe (it is
    /// the already-bumped leading identifier, disambiguated by the trailing
    /// `!`), so `'My Sheet':Sheet3!A1` parses as a span but the quoted-*end*
    /// form `Sheet1:'Q2 Data'!A1` remains a typed limitation (see the W078
    /// workset doc).
    fn at_sheet_span_tail(&self) -> bool {
        matches!(
            self.peek_significant_kinds(3).as_slice(),
            [TokenKind::Colon, TokenKind::Identifier, TokenKind::Bang]
        )
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
                | TokenKind::Dot
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

fn normalize_reference_selector_token(text: &str) -> String {
    text.trim_start_matches('@').to_ascii_uppercase()
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
