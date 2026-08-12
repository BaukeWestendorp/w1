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

        Ast { nodes: self.nodes }
    }

    fn parse_node(&mut self) -> Option<Node<'src>> {
        match self.peek_kind()? {
            TokenKind::At => match self.peek_next_kind() {
                Some(TokenKind::Ident("macro")) => Some(self.expect_macro()),
                Some(TokenKind::Ident(_)) => Some(self.parse_invoke()),
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
            Some(TokenKind::Comma) => Node::Text(","),
            Some(TokenKind::BlockOpen) => Node::Text("{{"),
            Some(TokenKind::Text(text)) => Node::Text(text),
            Some(TokenKind::Whitespace(ws)) => Node::Text(ws),
            Some(TokenKind::Variable(name)) => Node::Variable(Variable { name }),
            Some(TokenKind::BlockClose) => self.error("unexpected block close while parsing text"),
            None => self.error("unexpected EOF while parsing text"),
        }
    }

    fn expect_macro(&mut self) -> Node<'src> {
        self.expect(TokenKind::At);
        self.expect(TokenKind::Ident("macro"));
        let name = self.expect_ident("expected identifier after @macro");
        let params = self.parse_params();
        let template = self.parse_block();
        Node::Macro(Macro {
            name,
            params,
            template,
        })
    }

    fn parse_invoke(&mut self) -> Node<'src> {
        self.expect(TokenKind::At);
        let name = self.expect_ident("expected identifier after @");
        let args = self.parse_args();
        Node::Invokation(Invokation { name, args })
    }

    fn parse_params(&mut self) -> Vec<Variable<'src>> {
        let mut params = Vec::new();
        self.expect(TokenKind::ParenOpen);
        loop {
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

    fn parse_args(&mut self) -> Vec<Block<'src>> {
        let mut args = Vec::new();
        self.expect(TokenKind::ParenOpen);
        loop {
            if self.peek_kind() == Some(TokenKind::ParenClose) {
                break;
            }

            let block = self.parse_block();
            args.push(block);

            if self.peek_kind() == Some(TokenKind::Comma) {
                self.bump();
            }
        }
        self.expect(TokenKind::ParenClose);
        args
    }

    fn parse_block(&mut self) -> Block<'src> {
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

    fn eat(&mut self, expected: TokenKind<'src>) -> bool {
        if self.peek_kind() == Some(expected) {
            self.bump();
            true
        } else {
            false
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

    fn expect_ident(&mut self, message: &str) -> &'src str {
        match self.bump_kind() {
            Some(TokenKind::Ident(name)) => name,
            other => self.error(&format!(
                "{}: expected identifier, got {:?}",
                message, other
            )),
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
    pub nodes: Vec<Node<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'src> {
    Macro(Macro<'src>),
    Invokation(Invokation<'src>),
    Variable(Variable<'src>),
    Text(&'src str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Macro<'src> {
    pub name: &'src str,
    pub params: Vec<Variable<'src>>,
    pub template: Block<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Invokation<'src> {
    pub name: &'src str,
    pub args: Vec<Block<'src>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable<'src> {
    pub name: &'src str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block<'src> {
    pub nodes: Vec<Node<'src>>,
}
