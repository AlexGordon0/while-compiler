use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct LexError(String);

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LexError: {}", self.0)
    }
}

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

    fn lex_number(&mut self) -> Result<Token, LexError> {
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
            return Err(LexError(format!("{lexeme} exceeds i64 limit")));
        };
        Ok(Token::Num(n))
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
                    if self.peek() == Some('-') {
                        self.bump();
                        tokens.push(Token::Assn);
                    } else {
                        tokens.push(Token::LessThan);
                    }
                }
                '0'..='9' => tokens.push(self.lex_number()?),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        Lexer::new(input).lex().unwrap()
    }

    #[test]
    fn empty_input_is_just_end() {
        assert_eq!(lex(""), vec![Token::End]);
        assert_eq!(lex("  \n\t "), vec![Token::End]);
    }

    #[test]
    fn keywords() {
        assert_eq!(
            lex("if then else while do skip true false"),
            vec![
                Token::If,
                Token::Then,
                Token::Else,
                Token::While,
                Token::Do,
                Token::Skip,
                Token::True,
                Token::False,
                Token::End,
            ]
        );
    }

    #[test]
    fn identifiers() {
        assert_eq!(
            lex("x foo_bar a1 iff"),
            vec![
                Token::Id("x".to_string()),
                Token::Id("foo_bar".to_string()),
                Token::Id("a1".to_string()),
                Token::Id("iff".to_string()),
                Token::End,
            ]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            lex("0 42 9223372036854775807"),
            vec![
                Token::Num(0),
                Token::Num(42),
                Token::Num(i64::MAX),
                Token::End
            ]
        );
    }

    #[test]
    fn operators_and_punctuation() {
        assert_eq!(
            lex("+ - * = ! ( ) { } ; && || < <-"),
            vec![
                Token::Plus,
                Token::Minus,
                Token::Times,
                Token::Equals,
                Token::Not,
                Token::LeftParen,
                Token::RightParen,
                Token::LeftBracket,
                Token::RightBracket,
                Token::Semicolon,
                Token::And,
                Token::Or,
                Token::LessThan,
                Token::Assn,
                Token::End,
            ]
        );
    }

    #[test]
    fn no_whitespace_needed() {
        assert_eq!(
            lex("x<-x*2;y<1"),
            vec![
                Token::Id("x".to_string()),
                Token::Assn,
                Token::Id("x".to_string()),
                Token::Times,
                Token::Num(2),
                Token::Semicolon,
                Token::Id("y".to_string()),
                Token::LessThan,
                Token::Num(1),
                Token::End,
            ]
        );
    }

    #[test]
    fn less_than_then_negative_is_assignment() {
        // `<-` is always lexed as assignment, even where `< -` was intended.
        assert_eq!(
            lex("a <-1"),
            vec![
                Token::Id("a".to_string()),
                Token::Assn,
                Token::Num(1),
                Token::End
            ]
        );
    }

    #[test]
    fn number_overflow_is_error() {
        assert!(Lexer::new("99999999999999999999").lex().is_err());
    }

    #[test]
    fn single_ampersand_or_pipe_is_error() {
        assert!(Lexer::new("a & b").lex().is_err());
        assert!(Lexer::new("a | b").lex().is_err());
    }

    #[test]
    fn unexpected_characters_are_errors() {
        assert!(Lexer::new("X <- 1").lex().is_err());
        assert!(Lexer::new("x <- 1 / 2").lex().is_err());
        assert!(Lexer::new("_x <- 1").lex().is_err());
    }
}
