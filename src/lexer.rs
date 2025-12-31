#[derive(Debug)]
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
    Star,
    True,
    False,
    LessThan,
    Equals,
    And,
    Or,
    Not,
    Num(String),
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semicolon,
    Assn,
    End,
}

pub struct Lexer {
    text: String,
}

impl Lexer {
    pub fn new(text: String) -> Lexer {
        Lexer { text }
    }

    fn peek(&self) -> Option<char> {
        self.text.chars().next()
    }

    fn eat(&mut self, c: char) {
        if let Some(ch) = self.peek() {
            if c == ch {
                self.text.remove(0);
            } else {
                panic!("Expected {ch}, got {c}");
            }
        } else {
            panic!("Input is empty");
        }
    }

    fn lex_number(&mut self) -> Token {
        let mut lexeme = String::from("");
        while let Some(ch) = self.peek()
            && ch.is_ascii_digit()
        {
            self.eat(ch);
            lexeme.push(ch);
        }
        Token::Num(lexeme)
    }

    fn lex_kw_or_id(&mut self) -> Token {
        let mut lexeme = String::from("");
        while let Some(ch) = self.peek()
            && (ch.is_ascii_alphanumeric() || ch == '`')
        {
            self.eat(ch);
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

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some(ch) = self.peek() {
            match ch {
                '+' => {
                    self.eat('+');
                    tokens.push(Token::Plus);
                }
                '-' => {
                    self.eat('-');
                    tokens.push(Token::Minus);
                }
                '*' => {
                    self.eat('*');
                    tokens.push(Token::Star);
                }
                '=' => {
                    self.eat('=');
                    tokens.push(Token::Equals);
                }
                '!' => {
                    self.eat('!');
                    tokens.push(Token::Not);
                }
                '(' => {
                    self.eat('(');
                    tokens.push(Token::LeftParen);
                }
                ')' => {
                    self.eat(')');
                    tokens.push(Token::RightParen);
                }
                '{' => {
                    self.eat('{');
                    tokens.push(Token::LeftBracket);
                }
                '}' => {
                    self.eat('}');
                    tokens.push(Token::RightBracket);
                }
                ';' => {
                    self.eat(';');
                    tokens.push(Token::Semicolon);
                }
                '&' => {
                    self.eat('&');
                    self.eat('&');
                    tokens.push(Token::And);
                }
                '|' => {
                    self.eat('|');
                    self.eat('|');
                    tokens.push(Token::Or);
                }
                '<' => {
                    self.eat('<');
                    if let Some(ch1) = self.peek() {
                        match ch1 {
                            '-' => {
                                self.eat('-');
                                tokens.push(Token::Assn);
                            }
                            _ => tokens.push(Token::LessThan),
                        }
                    }
                }
                '0'..='9' => tokens.push(self.lex_number()),
                'a'..='z' => tokens.push(self.lex_kw_or_id()),
                ' ' => self.eat(ch),
                _ => panic!("Unexpected character: {ch}"),
            }
        }
        tokens.push(Token::End);
        tokens
    }
}
