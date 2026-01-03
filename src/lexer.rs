use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
#[allow(dead_code)]
pub struct LexError(String);

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    If,
    Then,
    Else,
    While,
    Do,
    Skip,
    Id(String),
    Plus,
    Minus,
    Times,
    True,
    False,
    LessThan,
    Equals,
    And,
    Or,
    Not,
    Num(i64),
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semicolon,
    Assn,
    End,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn bump(&mut self) {
        self.input.next();
    }

    fn lex_number(&mut self) -> Token {
        let mut lexeme = String::new();
        while let Some(ch) = self.peek()
            && ch.is_ascii_digit()
        {
            self.bump();
            lexeme.push(ch);
        }
        let n = if let Ok(n) = lexeme.parse::<i64>() {
            n
        } else {
            unreachable!()
        };
        Token::Num(n)
    }

    fn lex_kw_or_id(&mut self) -> Token {
        let mut lexeme = String::new();
        while let Some(ch) = self.peek()
            && (ch.is_ascii_alphanumeric() || ch == '_')
        {
            self.bump();
            lexeme.push(ch);
        }
        match lexeme.as_str() {
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "while" => Token::While,
            "do" => Token::Do,
            "skip" => Token::Skip,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Id(lexeme),
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = vec![];
        while let Some(ch) = self.peek() {
            match ch {
                '+' => {
                    self.bump();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    self.bump();
                    tokens.push(Token::Minus);
                }
                '*' => {
                    self.bump();
                    tokens.push(Token::Times);
                }
                '=' => {
                    self.bump();
                    tokens.push(Token::Equals);
                }
                '!' => {
                    self.bump();
                    tokens.push(Token::Not);
                }
                '(' => {
                    self.bump();
                    tokens.push(Token::LeftParen);
                }
                ')' => {
                    self.bump();
                    tokens.push(Token::RightParen);
                }
                '{' => {
                    self.bump();
                    tokens.push(Token::LeftBracket);
                }
                '}' => {
                    self.bump();
                    tokens.push(Token::RightBracket);
                }
                ';' => {
                    self.bump();
                    tokens.push(Token::Semicolon);
                }
                '&' => {
                    self.bump();
                    if Some('&') == self.peek() {
                        self.bump();
                    } else {
                        return Err(LexError("Expected &".to_string()));
                    }
                    tokens.push(Token::And);
                }
                '|' => {
                    self.bump();
                    if Some('|') == self.peek() {
                        self.bump();
                    } else {
                        return Err(LexError("Expected |".to_string()));
                    }
                    tokens.push(Token::Or);
                }
                '<' => {
                    self.bump();
                    if let Some(next_ch) = self.peek() {
                        match next_ch {
                            '-' => {
                                self.bump();
                                tokens.push(Token::Assn);
                            }
                            _ => tokens.push(Token::LessThan),
                        }
                    }
                }
                '0'..='9' => tokens.push(self.lex_number()),
                'a'..='z' => tokens.push(self.lex_kw_or_id()),
                ch if ch.is_whitespace() => self.bump(),
                _ => {
                    return Err(LexError(format!("Unexpected character: '{ch}'")));
                }
            }
        }
        tokens.push(Token::End);
        Ok(tokens)
    }
}
