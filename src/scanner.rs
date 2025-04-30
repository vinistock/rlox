use crate::token::*;

pub struct Scanner<'a> {
    tokens: Vec<Token>,
    errors: &'a mut Vec<String>,
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str, errors: &'a mut Vec<String>) -> Scanner<'a> {
        Scanner {
            tokens: Vec::new(),
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
            .push(Token::new(TokenType::Eof, "".to_string(), self.line));
    }

    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    fn scan_token(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        let char = self.advance(chars);

        match char {
            Some('(') => self.add_token(TokenType::LeftParen),
            Some(')') => self.add_token(TokenType::RightParen),
            Some('{') => self.add_token(TokenType::LeftBrace),
            Some('}') => self.add_token(TokenType::RightBrace),
            Some(',') => self.add_token(TokenType::Comma),
            Some('.') => self.add_token(TokenType::Dot),
            Some('-') => self.add_token(TokenType::Minus),
            Some('+') => self.add_token(TokenType::Plus),
            Some(';') => self.add_token(TokenType::Semicolon),
            Some('*') => self.add_token(TokenType::Star),
            Some('!') => {
                let token_type = if self.match_char('=', chars) {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type);
            }
            Some('=') => {
                let token_type = if self.match_char('=', chars) {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type);
            }
            Some('<') => {
                let token_type = if self.match_char('=', chars) {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token_type);
            }
            Some('>') => {
                let token_type = if self.match_char('=', chars) {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type);
            }
            Some('/') => {
                if self.match_char('/', chars) {
                    let comment = chars.take_while(|&c| c != '\n');
                    self.current += comment.map(|c| c.len_utf8()).sum::<usize>();
                    self.current += 1;
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            Some(' ') | Some('\r') | Some('\t') => {}
            Some('\n') => {
                self.line += 1;
            }
            Some('"') => self.string(chars),
            Some(c) if c.is_ascii_digit() => {
                self.number(chars);
            }
            Some(c) if c.is_alphanumeric() || c == '_' => {
                self.identifier(chars);
            }
            Some(c) => {
                self.errors.push(format!(
                    "Unexpected character '{}' at line {}",
                    c, self.line
                ));
            }
            None => {}
        }
    }

    fn identifier(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        while let Some(c) = chars.peek() {
            if c.is_alphanumeric() || *c == '_' {
                self.advance(chars);
            } else {
                break;
            }
        }

        let text = &self.source[self.start..self.current];

        match text {
            "and" => self.add_token(TokenType::And),
            "class" => self.add_token(TokenType::Class),
            "else" => self.add_token(TokenType::Else),
            "false" => self.add_token(TokenType::False),
            "for" => self.add_token(TokenType::For),
            "fun" => self.add_token(TokenType::Fun),
            "if" => self.add_token(TokenType::If),
            "nil" => self.add_token(TokenType::Nil),
            "or" => self.add_token(TokenType::Or),
            "print" => self.add_token(TokenType::Print),
            "return" => self.add_token(TokenType::Return),
            "super" => self.add_token(TokenType::Super),
            "this" => self.add_token(TokenType::This),
            "true" => self.add_token(TokenType::True),
            "var" => self.add_token(TokenType::Var),
            "while" => self.add_token(TokenType::While),
            _ => self.add_token(TokenType::Identifier),
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
        self.add_token_with_text(TokenType::Number, number_str.to_string());
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
                self.add_token_with_text(TokenType::String, string_value);
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

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, text, self.line));
    }

    fn add_token_with_text(&mut self, token_type: TokenType, text: String) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(source: &str) -> Vec<Token> {
        let mut errors = Vec::new();
        let mut scanner = Scanner::new(source, &mut errors);
        scanner.scan();
        scanner.into_tokens()
    }

    #[test]
    fn test_scanning_single_character_tokens() {
        let map = vec![
            ('(', TokenType::LeftParen),
            (')', TokenType::RightParen),
            ('{', TokenType::LeftBrace),
            ('}', TokenType::RightBrace),
            (',', TokenType::Comma),
            ('.', TokenType::Dot),
            ('-', TokenType::Minus),
            ('+', TokenType::Plus),
            (';', TokenType::Semicolon),
            ('*', TokenType::Star),
            ('/', TokenType::Slash),
        ];

        for (char, token_type) in map {
            let source = String::from(char);
            let tokens = scan(&source);
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].token_type, token_type);
        }
    }

    #[test]
    fn test_scanning_something_equal_tokens() {
        let map = vec![
            ("!".to_string(), TokenType::Bang),
            ("!=".to_string(), TokenType::BangEqual),
            ("=".to_string(), TokenType::Equal),
            ("==".to_string(), TokenType::EqualEqual),
            (">".to_string(), TokenType::Greater),
            (">=".to_string(), TokenType::GreaterEqual),
            ("<".to_string(), TokenType::Less),
            ("<=".to_string(), TokenType::LessEqual),
        ];

        for (source, token_type) in map {
            let tokens = scan(&source);
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].token_type, token_type);
        }
    }

    #[test]
    fn test_scanning_comments() {
        let source = "// some content hello\n".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn test_scanning_strings() {
        let source = "\"some string content\"".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].lexeme, "some string content".to_string());
    }

    #[test]
    fn test_scanning_numbers() {
        let source = "123".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].lexeme, "123".to_string());
    }

    #[test]
    fn test_scanning_numbers_with_fractional_values() {
        let source = "123.321".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].lexeme, "123.321".to_string());
    }

    #[test]
    fn test_scanning_identifiers() {
        let source = "iDentifier_".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].lexeme, "iDentifier_".to_string());
    }

    #[test]
    fn test_scanning_keywords() {
        let keywords = vec![
            ("and".to_string(), TokenType::And),
            ("class".to_string(), TokenType::Class),
            ("else".to_string(), TokenType::Else),
            ("false".to_string(), TokenType::False),
            ("for".to_string(), TokenType::For),
            ("fun".to_string(), TokenType::Fun),
            ("if".to_string(), TokenType::If),
            ("nil".to_string(), TokenType::Nil),
            ("or".to_string(), TokenType::Or),
            ("print".to_string(), TokenType::Print),
            ("return".to_string(), TokenType::Return),
            ("super".to_string(), TokenType::Super),
            ("this".to_string(), TokenType::This),
            ("true".to_string(), TokenType::True),
            ("var".to_string(), TokenType::Var),
            ("while".to_string(), TokenType::While),
        ];

        for (keyword, token_type) in keywords {
            let source = keyword.clone();
            let tokens = scan(&source);
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].token_type, token_type);
            assert_eq!(tokens[0].lexeme, keyword);
        }
    }
}
