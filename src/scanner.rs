use crate::token::*;

pub struct Scanner<'a> {
    tokens: &'a mut Vec<Token>,
    source: &'a String,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String, tokens: &'a mut Vec<Token>) -> Scanner<'a> {
        Scanner {
            tokens,
            source,
            start: 0,
            current: 0,
            line: 0,
        }
    }

    pub fn scan(&mut self) {
        let mut chars = self.source.chars();

        while self.current < self.source.len() {
            self.start = self.current;
            self.scan_token(&mut chars);
        }

        self.tokens
            .push(Token::new(TokenTypes::Eof, "".to_string(), self.line));
    }

    fn scan_token(&mut self, chars: &mut std::str::Chars) {
        let char = self.advance(chars);

        match char {
            '(' => self.tokens.push(Token::new(
                TokenTypes::LeftParen,
                char.to_string(),
                self.line,
            )),
            _ => eprintln!("Unexpected character: {}", char),
        }
    }

    fn advance(&mut self, chars: &mut std::str::Chars) -> char {
        let c = chars.next().unwrap_or('\0');
        self.current += c.len_utf8();
        c
    }
}

pub fn scan(source: &String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    Scanner::new(source, &mut tokens).scan();
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_left_parenthesis() {
        let source = String::from("(");
        let tokens = scan(&source);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenTypes::LeftParen);
    }
}
