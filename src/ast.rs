use std::collections::VecDeque;

use crate::token::{Token, TokenKind};

pub fn parse<'src>(src: &'src str, tokens: impl Iterator<Item = Token<'src>>) -> Ast<'src> {
    Parser::new(src, tokens).parse()
}

struct Parser<'src, I: Iterator<Item = Token<'src>>> {
    src: &'src str,
    tokens: I,
    buffered: VecDeque<Token<'src>>,
    nodes: Vec<Node<'src>>,
}

impl<'src, I: Iterator<Item = Token<'src>>> Parser<'src, I> {
    pub fn new(src: &'src str, tokens: I) -> Self {
        Self {
            src,
            tokens,
            buffered: VecDeque::new(),
            nodes: Vec::new(),
        }
    }

    fn fill(&mut self, n: usize) {
        while self.buffered.len() < n {
            let Some(token) = self.tokens.next() else {
                break;
            };
            self.buffered.push_back(token);
        }
    }

    fn peek(&mut self) -> Option<&Token<'src>> {
        self.fill(1);
        self.buffered.get(0)
    }

    fn peek_kind(&mut self) -> Option<TokenKind<'src>> {
        self.peek().map(|token| token.kind())
    }

    fn peek_next(&mut self) -> Option<&Token<'src>> {
        self.fill(2);
        self.buffered.get(1)
    }

    fn peek_next_kind(&mut self) -> Option<TokenKind<'src>> {
        self.peek_next().map(|token| token.kind())
    }

    fn bump(&mut self) -> Option<Token<'src>> {
        self.fill(1);
        self.buffered.pop_front()
    }

    fn bump_kind(&mut self) -> Option<TokenKind<'src>> {
        self.bump().map(|token| token.kind())
    }

    pub fn parse(mut self) -> Ast<'src> {
        loop {
            let Some(node) = self.parse_node() else { break };
            self.nodes.push(node);
        }

        Ast {
            root: Block { nodes: self.nodes },
        }
    }

    fn parse_node(&mut self) -> Option<Node<'src>> {
        match self.peek_kind()? {
            TokenKind::At => match self.peek_next_kind() {
                Some(TokenKind::Ident("define")) => Some(self.expect_define()),
                Some(TokenKind::Ident("for")) => Some(self.expect_for()),
                Some(TokenKind::Ident("macro")) => Some(self.expect_macro()),
                Some(TokenKind::Ident("call")) => Some(self.expect_call()),
                Some(TokenKind::Ident(other)) => {
                    self.error(&format!("unexpected identifier after @: {}", other))
                }
                _ => Some(self.parse_text()),
            },
            TokenKind::Variable(_) => match self.bump_kind() {
                Some(TokenKind::Variable(name)) => Some(Node::Variable(Variable { name })),
                _ => unreachable!(),
            },
            TokenKind::BlockClose => None,
            _ => Some(self.parse_text()),
        }
    }

    fn parse_text(&mut self) -> Node<'src> {
        match self.bump_kind() {
            Some(TokenKind::At) => Node::Text("@"),
            Some(TokenKind::Ident(text)) => Node::Text(text),
            Some(TokenKind::ParenOpen) => Node::Text("("),
            Some(TokenKind::ParenClose) => Node::Text(")"),
            Some(TokenKind::BracketOpen) => Node::Text("["),
            Some(TokenKind::BracketClose) => Node::Text("]"),
            Some(TokenKind::Comma) => Node::Text(","),
            Some(TokenKind::Equal) => Node::Text("="),
            Some(TokenKind::BlockOpen) => Node::Text("{{"),
            Some(TokenKind::Text(text)) => Node::Text(text),
            Some(TokenKind::Whitespace(ws)) => Node::Text(ws),
            Some(TokenKind::Variable(name)) => Node::Variable(Variable { name }),
            Some(TokenKind::BlockClose) => self.error("unexpected block close while parsing text"),
            None => self.error("unexpected EOF while parsing text"),
        }
    }

    fn expect_define(&mut self) -> Node<'src> {
        self.expect(TokenKind::At);
        self.expect(TokenKind::Ident("define"));
        self.skip_whitespace();
        let name = self.expect_var();
        self.skip_whitespace();
        self.expect(TokenKind::Equal);
        self.skip_whitespace();
        let list = self.expect_list_definition();

        Node::Define(Define { name, list })
    }

    fn expect_for(&mut self) -> Node<'src> {
        self.expect(TokenKind::At);
        self.expect(TokenKind::Ident("for"));
        self.skip_whitespace();
        let fields = self.expect_list();
        self.skip_whitespace();
        self.expect(TokenKind::Ident("in"));
        self.skip_whitespace();
        let list = self.expect_var();
        self.skip_whitespace();
        let template = self.expect_block();

        Node::For(For {
            list,
            fields,
            template,
        })
    }

    fn expect_macro(&mut self) -> Node<'src> {
        self.expect(TokenKind::At);
        self.expect(TokenKind::Ident("macro"));
        self.skip_whitespace();
        let name = self.expect_ident();
        self.skip_whitespace();
        let params = self.expect_params();
        self.skip_whitespace();
        let template = self.expect_block();
        Node::Macro(Macro {
            name,
            parameters: params,
            template,
        })
    }

    fn expect_call(&mut self) -> Node<'src> {
        self.expect(TokenKind::At);
        self.expect(TokenKind::Ident("call"));
        self.skip_whitespace();
        let name = self.expect_ident();
        self.skip_whitespace();
        let args = self.expect_args();
        Node::Call(Call {
            name,
            arguments: args,
        })
    }

    fn expect_params(&mut self) -> Vec<Variable<'src>> {
        let mut params = Vec::new();
        self.expect(TokenKind::ParenOpen);
        loop {
            self.skip_whitespace();
            if self.peek_kind() == Some(TokenKind::ParenClose) {
                break;
            }
            match self.bump_kind() {
                Some(TokenKind::Variable(var)) => params.push(Variable { name: var }),
                Some(TokenKind::Comma) => continue,
                Some(token) => self.error(&format!("expected variable or comma, got {:?}", token)),
                None => self.error("unexpected EOF while parsing parameters"),
            }
        }
        self.expect(TokenKind::ParenClose);
        params
    }

    fn expect_list(&mut self) -> Vec<Variable<'src>> {
        let mut params = Vec::new();
        self.expect(TokenKind::BracketOpen);
        loop {
            self.skip_whitespace();
            if self.peek_kind() == Some(TokenKind::BracketClose) {
                break;
            }
            match self.bump_kind() {
                Some(TokenKind::Variable(var)) => params.push(Variable { name: var }),
                Some(TokenKind::Comma) => continue,
                Some(token) => self.error(&format!("expected variable or comma, got {:?}", token)),
                None => self.error("unexpected EOF while parsing list"),
            }
        }
        self.expect(TokenKind::BracketClose);
        params
    }

    fn expect_args(&mut self) -> Vec<Block<'src>> {
        let mut args = Vec::new();
        self.expect(TokenKind::ParenOpen);
        loop {
            self.skip_whitespace();
            if self.peek_kind() == Some(TokenKind::ParenClose) {
                break;
            }
            let block = self.expect_block();
            args.push(block);
            self.skip_whitespace();
            if self.peek_kind() == Some(TokenKind::Comma) {
                self.bump();
            }
        }
        self.expect(TokenKind::ParenClose);
        args
    }

    fn expect_block(&mut self) -> Block<'src> {
        let mut block = Block { nodes: Vec::new() };
        self.expect(TokenKind::BlockOpen);
        while let Some(token) = self.peek_kind() {
            if token == TokenKind::BlockClose {
                break;
            }
            let node = self.parse_node().expect("expected node inside block");
            block.nodes.push(node);
        }
        self.expect(TokenKind::BlockClose);
        block
    }

    fn expect_list_definition(&mut self) -> ListDefinition<'src> {
        let mut objects = Vec::new();
        self.expect(TokenKind::BracketOpen);
        loop {
            self.skip_whitespace();
            if self.peek_kind() == Some(TokenKind::BracketClose) {
                break;
            }
            let object = self.expect_object();
            objects.push(object);
            self.skip_whitespace();
            if self.peek_kind() == Some(TokenKind::Comma) {
                self.bump();
            }
        }
        self.expect(TokenKind::BracketClose);
        ListDefinition { objects }
    }

    fn expect_object(&mut self) -> Object<'src> {
        let mut fields = Vec::new();
        self.expect(TokenKind::BracketOpen);
        loop {
            self.skip_whitespace();
            if self.peek_kind() == Some(TokenKind::BracketClose) {
                break;
            }
            let field = self.expect_field();
            fields.push(field);
            self.skip_whitespace();
            if self.peek_kind() == Some(TokenKind::Comma) {
                self.bump();
            }
        }
        self.expect(TokenKind::BracketClose);
        Object { fields }
    }

    fn expect_field(&mut self) -> Field<'src> {
        let name = self.expect_var();
        self.skip_whitespace();
        self.expect(TokenKind::Equal);
        self.skip_whitespace();
        let value = self.expect_block();
        Field { name, value }
    }

    fn expect_ident(&mut self) -> &'src str {
        match self.bump_kind() {
            Some(TokenKind::Ident(name)) => name,
            other => self.error(&format!("expected identifier, got {:?}", other)),
        }
    }

    fn expect_var(&mut self) -> Variable<'src> {
        match self.bump_kind() {
            Some(TokenKind::Variable(name)) => Variable { name },
            other => self.error(&format!("expected variable, got {:?}", other)),
        }
    }

    fn expect(&mut self, expected: TokenKind<'src>) {
        if !self.eat(expected) {
            let got = self
                .peek_kind()
                .map(|kind| kind.to_string())
                .unwrap_or("<EOF>".to_string());
            self.error(&format!("expected token: '{}', got '{}'", expected, got));
        }
    }

    fn eat(&mut self, expected: TokenKind<'src>) -> bool {
        if self.peek_kind() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_kind(), Some(TokenKind::Whitespace(_))) {
            self.bump();
        }
    }

    fn error(&mut self, message: &str) -> ! {
        match self.peek().map(|t| t.span().clone()) {
            Some(span) => panic!("ERROR: {}\n > {}", message, &self.src[span]),
            _ => panic!("{} at EOF", message),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ast<'src> {
    pub root: Block<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'src> {
    Define(Define<'src>),
    For(For<'src>),
    Macro(Macro<'src>),
    Call(Call<'src>),
    Variable(Variable<'src>),
    List(ListDefinition<'src>),
    Text(&'src str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Define<'src> {
    pub name: Variable<'src>,
    // FIXME: Would be noice if this could be a general `Value` so you can define constants too.
    pub list: ListDefinition<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct For<'src> {
    pub list: Variable<'src>,
    pub fields: Vec<Variable<'src>>,
    pub template: Block<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Macro<'src> {
    pub name: &'src str,
    pub parameters: Vec<Variable<'src>>,
    pub template: Block<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'src> {
    pub name: &'src str,
    pub arguments: Vec<Block<'src>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable<'src> {
    pub name: &'src str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block<'src> {
    pub nodes: Vec<Node<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListDefinition<'src> {
    pub objects: Vec<Object<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object<'src> {
    pub fields: Vec<Field<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<'src> {
    pub name: Variable<'src>,
    pub value: Block<'src>,
}
