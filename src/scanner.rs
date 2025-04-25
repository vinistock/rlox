use crate::token::*;

pub struct Scanner<'a> {
    tokens: &'a mut Vec<Token>,
    errors: &'a mut Vec<String>,
    source: &'a String,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(
        source: &'a String,
        tokens: &'a mut Vec<Token>,
        errors: &'a mut Vec<String>,
    ) -> Scanner<'a> {
        Scanner {
            tokens,
            errors,
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan(&mut self) {
        let mut chars = self.source.chars().peekable();

        while self.current < self.source.len() {
            self.start = self.current;
            self.scan_token(&mut chars);
        }

        self.tokens
            .push(Token::new(TokenTypes::Eof, "".to_string(), self.line));
    }

    fn scan_token(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        let char = self.advance(chars);
        if char.is_none() {
            return;
        }

        let character = char.unwrap();

        match character {
            '(' => self.add_token(TokenTypes::LeftParen),
            ')' => self.add_token(TokenTypes::RightParen),
            '{' => self.add_token(TokenTypes::LeftBrace),
            '}' => self.add_token(TokenTypes::RightBrace),
            ',' => self.add_token(TokenTypes::Comma),
            '.' => self.add_token(TokenTypes::Dot),
            '-' => self.add_token(TokenTypes::Minus),
            '+' => self.add_token(TokenTypes::Plus),
            ';' => self.add_token(TokenTypes::Semicolon),
            '*' => self.add_token(TokenTypes::Star),
            '!' => {
                let token_type = if self.match_char('=', chars) {
                    TokenTypes::BangEqual
                } else {
                    TokenTypes::Bang
                };
                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.match_char('=', chars) {
                    TokenTypes::EqualEqual
                } else {
                    TokenTypes::Equal
                };
                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.match_char('=', chars) {
                    TokenTypes::LessEqual
                } else {
                    TokenTypes::Less
                };
                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.match_char('=', chars) {
                    TokenTypes::GreaterEqual
                } else {
                    TokenTypes::Greater
                };
                self.add_token(token_type);
            }
            '/' => {
                if self.match_char('/', chars) {
                    let comment = chars.take_while(|&c| c != '\n');
                    self.current += comment.map(|c| c.len_utf8()).sum::<usize>();
                    self.current += 1;
                } else {
                    self.add_token(TokenTypes::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => self.string(chars),
            _ => {
                if character.is_ascii_digit() {
                    self.number(chars);
                } else {
                    self.errors.push(format!(
                        "Unexpected character '{}' at line {}",
                        character, self.line
                    ));
                }
            }
        }
    }

    fn number(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        loop {
            match chars.peek() {
                Some(c) if c.is_ascii_digit() => {
                    self.advance(chars);
                }
                Some(_) | None => break,
            }
        }

        if self.source[self.current..].starts_with('.') {
            let next_char = self.source[self.current + 1..].chars().next();
            if next_char.is_some() && next_char.unwrap().is_ascii_digit() {
                self.advance(chars);
                let digits = chars
                    .take_while(|&c| c.is_ascii_digit())
                    .collect::<String>();
                self.current += digits.len();
            }
        }

        let number_str = &self.source[self.start..self.current];
        self.add_token_with_text(TokenTypes::Number, number_str.to_string());
    }

    fn string(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        let mut string_value = String::new();

        loop {
            match chars.peek() {
                Some(&'"') => break,
                Some(&'\n') => {
                    string_value.push('\n');
                    self.line += 1;
                    self.current += 1;
                    chars.next();
                }
                Some(_) => {
                    if let Some(c) = chars.next() {
                        string_value.push(c);
                        self.current += c.len_utf8();
                    }
                }
                None => break,
            }
        }

        let closing_quote = self.advance(chars);

        match closing_quote {
            Some(quote) => {
                self.current += quote.len_utf8();
                self.add_token_with_text(TokenTypes::String, string_value);
            }
            None => {
                self.errors
                    .push(format!("Unterminated string at line {}", self.line));
            }
        }
    }

    fn match_char(
        &mut self,
        expected: char,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        if self.current >= self.source.len() {
            return false;
        }

        match chars.peek() {
            Some(&c) if c == expected => {
                self.advance(chars);
                true
            }
            _ => false,
        }
    }

    fn add_token(&mut self, token_type: TokenTypes) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, text, self.line));
    }

    fn add_token_with_text(&mut self, token_type: TokenTypes, text: String) {
        self.tokens.push(Token::new(token_type, text, self.line));
    }

    fn advance(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<char> {
        match chars.next() {
            Some(c) => {
                self.current += c.len_utf8();
                Some(c)
            }
            None => None,
        }
    }
}

pub fn scan(source: &String) -> (Vec<Token>, Vec<String>) {
    let mut tokens: Vec<Token> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    Scanner::new(source, &mut tokens, &mut errors).scan();
    (tokens, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanning_single_character_tokens() {
        let map = vec![
            ('(', TokenTypes::LeftParen),
            (')', TokenTypes::RightParen),
            ('{', TokenTypes::LeftBrace),
            ('}', TokenTypes::RightBrace),
            (',', TokenTypes::Comma),
            ('.', TokenTypes::Dot),
            ('-', TokenTypes::Minus),
            ('+', TokenTypes::Plus),
            (';', TokenTypes::Semicolon),
            ('*', TokenTypes::Star),
            ('/', TokenTypes::Slash),
        ];

        for (char, token_type) in map {
            let source = String::from(char);
            let (tokens, errors) = scan(&source);
            assert!(errors.is_empty());
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].token_type, token_type);
        }
    }

    #[test]
    fn test_scanning_something_equal_tokens() {
        let map = vec![
            ("!".to_string(), TokenTypes::Bang),
            ("!=".to_string(), TokenTypes::BangEqual),
            ("=".to_string(), TokenTypes::Equal),
            ("==".to_string(), TokenTypes::EqualEqual),
            (">".to_string(), TokenTypes::Greater),
            (">=".to_string(), TokenTypes::GreaterEqual),
            ("<".to_string(), TokenTypes::Less),
            ("<=".to_string(), TokenTypes::LessEqual),
        ];

        for (source, token_type) in map {
            let (tokens, errors) = scan(&source);
            assert!(errors.is_empty());
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].token_type, token_type);
        }
    }

    #[test]
    fn test_scanning_comments() {
        let source = "// some content hello\n".to_string();
        let (tokens, errors) = scan(&source);
        assert!(errors.is_empty());
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenTypes::Eof);
    }

    #[test]
    fn test_scanning_strings() {
        let source = "\"some string content\"".to_string();
        let (tokens, errors) = scan(&source);
        assert!(errors.is_empty());
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenTypes::String);
        assert_eq!(tokens[0].lexeme, "some string content".to_string());
    }

    #[test]
    fn test_scanning_numbers() {
        let source = "123".to_string();
        let (tokens, errors) = scan(&source);
        assert!(errors.is_empty());
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenTypes::Number);
        assert_eq!(tokens[0].lexeme, "123".to_string());
    }

    #[test]
    fn test_scanning_numbers_with_fractional_values() {
        let source = "123.321".to_string();
        let (tokens, errors) = scan(&source);
        assert!(errors.is_empty());
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenTypes::Number);
        assert_eq!(tokens[0].lexeme, "123.321".to_string());
    }
}
