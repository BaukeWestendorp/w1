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
            return Some(Token::BlockOpen);
        } else if remaining.starts_with("}}") {
            self.pos += 2;
            self.block_depth = self.block_depth.saturating_sub(1);
            return Some(Token::BlockClose);
        }

        let first_char = remaining.chars().next().unwrap();

        match first_char {
            '@' => {
                self.pos += 1;
                Some(Token::At)
            }
            '(' => {
                self.pos += 1;
                Some(Token::ParenOpen)
            }
            ')' => {
                self.pos += 1;
                Some(Token::ParenClose)
            }
            ',' => {
                self.pos += 1;
                Some(Token::Comma)
            }
            '$' => {
                let end = remaining[1..]
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .map_or(remaining.len(), |i| i + 1);
                let var_name = &remaining[1..end];
                self.pos += end;
                Some(Token::Variable(var_name))
            }
            _ if first_char.is_alphanumeric() || first_char == '_' => {
                let end = remaining
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(remaining.len());
                let ident = &remaining[..end];
                self.pos += end;
                Some(Token::Ident(ident))
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
                Some(Token::Text(text))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token<'src> {
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

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::At => write!(f, "@"),
            Token::Ident(ident) => write!(f, "{}", ident),
            Token::Variable(var) => write!(f, "${}", var),
            Token::ParenOpen => write!(f, "("),
            Token::ParenClose => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::BlockOpen => write!(f, "{{"),
            Token::BlockClose => write!(f, "}}"),
            Token::Text(text) => write!(f, "{}", text),
        }
    }
}
