pub fn tokenize<'src>(src: &'src str) -> impl Iterator<Item = Token<'src>> {
    Tokenizer {
        src,
        pos: 0,
        block_depth: 0,
    }
}

struct Tokenizer<'src> {
    src: &'src str,
    pos: usize,
    block_depth: usize,
}

impl Tokenizer<'_> {
    pub fn span(&self) -> std::ops::Range<usize> {
        let start = self.pos;
        let end = self.src.len();
        start..end
    }
}

impl<'src> Iterator for Tokenizer<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        // Strip whitespace only when we are outside of a {{ ... }} block
        if self.block_depth == 0 {
            let remainder = &self.src[self.pos..];
            let trimmed = remainder.trim_start();
            self.pos += remainder.len() - trimmed.len();
        }

        if self.pos >= self.src.len() {
            return None;
        }

        let remaining = &self.src[self.pos..];

        if remaining.starts_with("{{") {
            self.pos += 2;
            self.block_depth += 1;
            return Some(Token::new(TokenKind::BlockOpen, self.span()));
        } else if remaining.starts_with("}}") {
            self.pos += 2;
            self.block_depth = self.block_depth.saturating_sub(1);
            return Some(Token::new(TokenKind::BlockClose, self.span()));
        }

        let first_char = remaining.chars().next().unwrap();

        match first_char {
            '@' => {
                self.pos += 1;
                Some(Token::new(TokenKind::At, self.span()))
            }
            '(' => {
                self.pos += 1;
                Some(Token::new(TokenKind::ParenOpen, self.span()))
            }
            ')' => {
                self.pos += 1;
                Some(Token::new(TokenKind::ParenClose, self.span()))
            }
            ',' => {
                self.pos += 1;
                Some(Token::new(TokenKind::Comma, self.span()))
            }
            '$' => {
                let end = remaining[1..]
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .map_or(remaining.len(), |i| i + 1);
                let var_name = &remaining[1..end];
                self.pos += end;
                Some(Token::new(TokenKind::Variable(var_name), self.span()))
            }
            _ if first_char.is_alphanumeric() || first_char == '_' => {
                let end = remaining
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(remaining.len());
                let ident = &remaining[..end];
                self.pos += end;
                Some(Token::new(TokenKind::Ident(ident), self.span()))
            }
            _ => {
                // Collect text until we hit a character that starts another valid token.
                // This approach prevents infinite loops on single braces '{' or '}'.
                let end = remaining
                    .char_indices()
                    .find_map(|(i, c)| {
                        if matches!(c, '@' | '$' | '(' | ')' | ',') {
                            Some(i)
                        } else if c == '{' && remaining[i..].starts_with("{{") {
                            Some(i)
                        } else if c == '}' && remaining[i..].starts_with("}}") {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(remaining.len());

                let text = &remaining[..end];
                self.pos += end;
                Some(Token::new(TokenKind::Text(text), self.span()))
            }
        }
    }
}

#[derive(Debug)]
pub struct Token<'src> {
    kind: TokenKind<'src>,
    span: std::ops::Range<usize>,
}

impl<'src> Token<'src> {
    pub fn new(kind: TokenKind<'src>, span: std::ops::Range<usize>) -> Self {
        Self { kind, span }
    }

    pub fn kind(&self) -> TokenKind<'src> {
        self.kind
    }

    pub fn span(&self) -> &std::ops::Range<usize> {
        &self.span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind<'src> {
    At,
    Ident(&'src str),
    Variable(&'src str),
    ParenOpen,
    ParenClose,
    Comma,
    BlockOpen,
    BlockClose,
    Text(&'src str),
}

impl std::fmt::Display for TokenKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::At => write!(f, "@"),
            TokenKind::Ident(ident) => write!(f, "{}", ident),
            TokenKind::Variable(var) => write!(f, "${}", var),
            TokenKind::ParenOpen => write!(f, "("),
            TokenKind::ParenClose => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::BlockOpen => write!(f, "{{"),
            TokenKind::BlockClose => write!(f, "}}"),
            TokenKind::Text(text) => write!(f, "{}", text),
            TokenKind::Whitespace(_) => write!(f, "<whitespace>"),
        }
    }
}
