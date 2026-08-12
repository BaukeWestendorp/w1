use std::collections::VecDeque;

use crate::token::Token;

pub fn parse<'src>(tokens: impl Iterator<Item = Token<'src>>) -> Ast<'src> {
    Parser::new(tokens).parse()
}

struct Parser<'src, I: Iterator<Item = Token<'src>>> {
    tokens: I,
    buffered: VecDeque<Token<'src>>,
    nodes: Vec<Node<'src>>,
}

impl<'src, I: Iterator<Item = Token<'src>>> Parser<'src, I> {
    pub fn new(tokens: I) -> Self {
        Self {
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

    fn peek_next(&mut self) -> Option<&Token<'src>> {
        self.fill(2);
        self.buffered.get(1)
    }

    fn bump(&mut self) -> Option<Token<'src>> {
        self.fill(1);
        self.buffered.pop_front()
    }

    pub fn parse(mut self) -> Ast<'src> {
        loop {
            let Some(node) = self.parse_node() else { break };
            self.nodes.push(node);
        }

        Ast { nodes: self.nodes }
    }

    fn parse_node(&mut self) -> Option<Node<'src>> {
        match self.peek()? {
            Token::At => match self.peek_next() {
                Some(Token::Ident("macro")) => Some(self.expect_macro()),
                Some(Token::Ident(_)) => Some(self.parse_invoke()),
                _ => Some(self.parse_text()),
            },
            Token::Variable(_) => match self.bump() {
                Some(Token::Variable(name)) => Some(Node::Variable(Variable { name })),
                _ => unreachable!(),
            },
            Token::BlockClose => None,
            _ => Some(self.parse_text()),
        }
    }

    fn parse_text(&mut self) -> Node<'src> {
        match self.bump() {
            Some(Token::At) => Node::Text("@"),
            Some(Token::Ident(text)) => Node::Text(text),
            Some(Token::ParenOpen) => Node::Text("("),
            Some(Token::ParenClose) => Node::Text(")"),
            Some(Token::Comma) => Node::Text(","),
            Some(Token::BlockOpen) => Node::Text("{{"),
            Some(Token::Text(text)) => Node::Text(text),
            Some(Token::Variable(name)) => Node::Variable(Variable { name }),
            Some(Token::BlockClose) => panic!("unexpected block close while parsing text"),
            None => panic!("unexpected EOF while parsing text"),
        }
    }

    fn expect_macro(&mut self) -> Node<'src> {
        self.expect(Token::At);
        self.expect(Token::Ident("macro"));

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
        self.expect(Token::At);

        let name = self.expect_ident("expected identifier after @");
        let args = self.parse_args();

        Node::Invokation(Invokation { name, args })
    }

    fn parse_params(&mut self) -> Vec<Variable<'src>> {
        let mut params = Vec::new();
        self.expect(Token::ParenOpen);
        loop {
            if self.peek() == Some(&Token::ParenClose) {
                break;
            }

            match self.bump() {
                Some(Token::Variable(var)) => params.push(Variable { name: var }),
                Some(Token::Comma) => continue,
                Some(token) => panic!("expected variable or comma, got {token:?}"),
                None => panic!("expected variable or comma, got EOF"),
            }
        }
        self.expect(Token::ParenClose);
        params
    }

    fn parse_args(&mut self) -> Vec<Block<'src>> {
        let mut args = Vec::new();
        self.expect(Token::ParenOpen);
        loop {
            if self.peek() == Some(&Token::ParenClose) {
                break;
            }

            let block = self.parse_block();
            args.push(block);

            if self.peek() == Some(&Token::Comma) {
                self.bump();
            }
        }
        self.expect(Token::ParenClose);
        args
    }

    fn parse_block(&mut self) -> Block<'src> {
        let mut block = Block { nodes: Vec::new() };
        self.expect(Token::BlockOpen);
        while let Some(token) = self.peek() {
            if *token == Token::BlockClose {
                break;
            }
            let node = self.parse_node().expect("expected node inside block");
            block.nodes.push(node);
        }
        self.expect(Token::BlockClose);
        block
    }

    fn eat(&mut self, expected: &Token<'src>) -> bool {
        if self.peek() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: Token<'src>) {
        assert!(
            self.eat(&expected),
            "expected {expected:?}, got {:?}",
            self.peek()
        );
    }

    fn expect_ident(&mut self, message: &str) -> &'src str {
        match self.bump() {
            Some(Token::Ident(name)) => name,
            other => panic!("{message}, got {other:?}"),
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
