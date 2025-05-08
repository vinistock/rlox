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

        self.tokens.push(Token::Eof);
    }

    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    fn scan_token(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        let char = self.advance(chars);

        match char {
            Some('(') => self.tokens.push(Token::LeftParen { line: self.line }),
            Some(')') => self.tokens.push(Token::RightParen { line: self.line }),
            Some('{') => self.tokens.push(Token::LeftBrace { line: self.line }),
            Some('}') => self.tokens.push(Token::RightBrace { line: self.line }),
            Some(',') => self.tokens.push(Token::Comma { line: self.line }),
            Some('.') => self.tokens.push(Token::Dot { line: self.line }),
            Some('-') => self.tokens.push(Token::Minus { line: self.line }),
            Some('+') => self.tokens.push(Token::Plus { line: self.line }),
            Some(';') => self.tokens.push(Token::Semicolon { line: self.line }),
            Some('*') => self.tokens.push(Token::Star { line: self.line }),
            Some('!') => {
                let token = if self.match_char('=', chars) {
                    Token::BangEqual { line: self.line }
                } else {
                    Token::Bang { line: self.line }
                };
                self.tokens.push(token);
            }
            Some('=') => {
                let token = if self.match_char('=', chars) {
                    Token::EqualEqual { line: self.line }
                } else {
                    Token::Equal { line: self.line }
                };
                self.tokens.push(token);
            }
            Some('<') => {
                let token = if self.match_char('=', chars) {
                    Token::LessEqual { line: self.line }
                } else {
                    Token::Less { line: self.line }
                };
                self.tokens.push(token);
            }
            Some('>') => {
                let token = if self.match_char('=', chars) {
                    Token::GreaterEqual { line: self.line }
                } else {
                    Token::Greater { line: self.line }
                };
                self.tokens.push(token);
            }
            Some('/') => {
                if self.match_char('/', chars) {
                    let comment = chars.take_while(|&c| c != '\n');
                    self.current += comment.map(|c| c.len_utf8()).sum::<usize>();
                    self.current += 1;
                } else {
                    self.tokens.push(Token::Slash { line: self.line });
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
            "and" => self.tokens.push(Token::And { line: self.line }),
            "class" => self.tokens.push(Token::Class { line: self.line }),
            "else" => self.tokens.push(Token::Else { line: self.line }),
            "false" => self.tokens.push(Token::False {
                line: self.line,
                value: false,
            }),
            "for" => self.tokens.push(Token::For { line: self.line }),
            "fun" => self.tokens.push(Token::Fun { line: self.line }),
            "if" => self.tokens.push(Token::If { line: self.line }),
            "nil" => self.tokens.push(Token::Nil { line: self.line }),
            "or" => self.tokens.push(Token::Or { line: self.line }),
            "print" => self.tokens.push(Token::Print { line: self.line }),
            "return" => self.tokens.push(Token::Return { line: self.line }),
            "super" => self.tokens.push(Token::Super { line: self.line }),
            "this" => self.tokens.push(Token::This { line: self.line }),
            "true" => self.tokens.push(Token::True {
                line: self.line,
                value: true,
            }),
            "var" => self.tokens.push(Token::Var { line: self.line }),
            "while" => self.tokens.push(Token::While { line: self.line }),
            _ => self.tokens.push(Token::Identifier(Identifier {
                line: self.line,
                value: text.to_string(),
            })),
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
        self.tokens.push(Token::Number {
            line: self.line,
            value: number_str.parse().unwrap(),
        });
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
                self.tokens.push(Token::String {
                    line: self.line,
                    value: string_value,
                });
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
            ('(', Token::LeftParen { line: 1 }),
            (')', Token::RightParen { line: 1 }),
            ('{', Token::LeftBrace { line: 1 }),
            ('}', Token::RightBrace { line: 1 }),
            (',', Token::Comma { line: 1 }),
            ('.', Token::Dot { line: 1 }),
            ('-', Token::Minus { line: 1 }),
            ('+', Token::Plus { line: 1 }),
            (';', Token::Semicolon { line: 1 }),
            ('*', Token::Star { line: 1 }),
            ('/', Token::Slash { line: 1 }),
        ];

        for (char, token) in map {
            let source = String::from(char);
            let tokens = scan(&source);
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0], token);
        }
    }

    #[test]
    fn test_scanning_something_equal_tokens() {
        let map = vec![
            ("!".to_string(), Token::Bang { line: 1 }),
            ("!=".to_string(), Token::BangEqual { line: 1 }),
            ("=".to_string(), Token::Equal { line: 1 }),
            ("==".to_string(), Token::EqualEqual { line: 1 }),
            (">".to_string(), Token::Greater { line: 1 }),
            (">=".to_string(), Token::GreaterEqual { line: 1 }),
            ("<".to_string(), Token::Less { line: 1 }),
            ("<=".to_string(), Token::LessEqual { line: 1 }),
        ];

        for (source, token) in map {
            let tokens = scan(&source);
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0], token);
        }
    }

    #[test]
    fn test_scanning_comments() {
        let source = "// some content hello\n".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Eof);
    }

    #[test]
    fn test_scanning_strings() {
        let source = "\"some string content\"".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0],
            Token::String {
                value: "some string content".to_string(),
                line: 1
            }
        );
    }

    #[test]
    fn test_scanning_numbers() {
        let source = "123".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0],
            Token::Number {
                value: 123.0,
                line: 1
            }
        );
    }

    #[test]
    fn test_scanning_numbers_with_fractional_values() {
        let source = "123.321".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0],
            Token::Number {
                value: 123.321,
                line: 1
            }
        );
    }

    #[test]
    fn test_scanning_identifiers() {
        let source = "iDentifier_".to_string();
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0],
            Token::Identifier(Identifier {
                value: "iDentifier_".to_string(),
                line: 1
            })
        );
    }

    #[test]
    fn test_scanning_keywords() {
        let keywords = vec![
            ("and".to_string(), Token::And { line: 1 }),
            ("class".to_string(), Token::Class { line: 1 }),
            ("else".to_string(), Token::Else { line: 1 }),
            (
                "false".to_string(),
                Token::False {
                    value: false,
                    line: 1,
                },
            ),
            ("for".to_string(), Token::For { line: 1 }),
            ("fun".to_string(), Token::Fun { line: 1 }),
            ("if".to_string(), Token::If { line: 1 }),
            ("nil".to_string(), Token::Nil { line: 1 }),
            ("or".to_string(), Token::Or { line: 1 }),
            ("print".to_string(), Token::Print { line: 1 }),
            ("return".to_string(), Token::Return { line: 1 }),
            ("super".to_string(), Token::Super { line: 1 }),
            ("this".to_string(), Token::This { line: 1 }),
            (
                "true".to_string(),
                Token::True {
                    value: true,
                    line: 1,
                },
            ),
            ("var".to_string(), Token::Var { line: 1 }),
            ("while".to_string(), Token::While { line: 1 }),
        ];

        for (keyword, token) in keywords {
            let source = keyword.clone();
            let tokens = scan(&source);
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0], token);
        }
    }
}
