use crate::lexer::Token;
use std::{iter::Peekable, slice::Iter};

#[derive(Debug)]
pub struct ParseError {
    expected: String,
    found: Option<Token>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(token) = &self.found {
            write!(f, "ParseError: Expected: [{}], Found: {token:?}", self.expected)
        } else {
            write!(f, "ParseError: Unexpected end of token input")
        }
    }
}

pub struct Parser<'a> {
    input: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [Token]) -> Self {
        Self {
            input: input.iter().peekable(),
        }
    }

    fn error(&mut self, expected: impl Into<String>) -> ParseError {
        ParseError {
            expected: expected.into(),
            found: self.peek().cloned(),
        }
    }

    fn eat(&mut self, tk: Token) -> Result<(), ParseError> {
        if let Some(tk1) = self.peek()
            && *tk1 == tk
        {
            self.input.next();
            Ok(())
        } else {
            Err(self.error(format!("{tk:?}")))
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.input.peek().copied()
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        self.prog()?;
        Ok(())
    }

    fn prog(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::If | Token::While | Token::Skip | Token::Id(..) | Token::LeftBracket => {
                    self.stmt()?;
                    self.stmts()?;
                    self.eat(Token::End)?;
                }
                _ => return Err(self.error("if, while, skip, id, (")),
            }
        };
        Ok(())
    }

    fn stmt(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::If => {
                    self.eat(Token::If)?;
                    self.bexp()?;
                    self.eat(Token::Then)?;
                    self.stmt()?;
                    self.eat(Token::Else)?;
                    self.stmt()?;
                }
                Token::While => {
                    self.eat(Token::While)?;
                    self.bexp()?;
                    self.eat(Token::Do)?;
                    self.stmt()?;
                }
                Token::Skip => {
                    self.eat(Token::Skip)?;
                }
                Token::Id(s) => {
                    let name = s.clone();
                    self.eat(Token::Id(name))?;
                    self.eat(Token::Assn)?;
                    self.aexp()?;
                }
                Token::LeftBracket => {
                    self.eat(Token::LeftBracket)?;
                    self.stmt()?;
                    self.stmts()?;
                    self.eat(Token::RightBracket)?;
                }
                _ => return Err(self.error("if, while, skip, id, (")),
            }
        }
        Ok(())
    }

    fn stmts(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::End | Token::RightBracket => (),
                Token::Semicolon => {
                    self.eat(Token::Semicolon)?;
                    self.stmt()?;
                    self.stmts()?;
                }
                _ => return Err(self.error("$, ), ;")),
            }
        }

        Ok(())
    }

    fn bexp(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Id(..)
                | Token::Not
                | Token::Num(..)
                | Token::True
                | Token::False
                | Token::LeftParen => {
                    self.bfac()?;
                    self.bexps()?;
                }
                _ => return Err(self.error("id, not, num, true, false, (")),
            }
        }

        Ok(())
    }

    fn bexps(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Then | Token::Do | Token::RightParen => (),
                Token::Or => {
                    self.eat(Token::Or)?;
                    self.bfac()?;
                    self.bexps()?;
                }
                _ => return Err(self.error("then, do, ), or")),
            }
        }

        Ok(())
    }

    fn bfac(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Id(..)
                | Token::Not
                | Token::Num(..)
                | Token::True
                | Token::False
                | Token::LeftParen => {
                    self.bneg()?;
                    self.bfacs()?;
                }
                _ => return Err(self.error("id, not, num, true, false, (")),
            }
        }

        Ok(())
    }

    fn bfacs(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Then | Token::Do | Token::Or | Token::RightParen => (),
                Token::And => {
                    self.eat(Token::And)?;
                    self.bneg()?;
                    self.bfacs()?;
                }
                _ => return Err(self.error("then, do, or, ), and")),
            }
        }

        Ok(())
    }

    fn bneg(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Id(..) | Token::Num(..) | Token::True | Token::False | Token::LeftParen => {
                    self.brel()?
                }
                Token::Not => {
                    self.eat(Token::Not)?;
                    self.bneg()?;
                }
                _ => return Err(self.error("id, num, true, false, (, not")),
            }
        }

        Ok(())
    }

    fn brel(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Id(..) | Token::Num(..) | Token::True | Token::False | Token::LeftParen => {
                    self.aexp()?;
                    self.brels()?;
                }
                _ => return Err(self.error("id, num, true, false, (")),
            }
        }

        Ok(())
    }

    fn brels(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Then | Token::Do | Token::Or | Token::And | Token::RightParen => (),
                Token::LessThan => {
                    self.eat(Token::LessThan)?;
                    self.aexp()?;
                }
                Token::Equals => {
                    self.eat(Token::Equals)?;
                    self.aexp()?;
                }
                _ => return Err(self.error("then, do, or, and, ), <, =")),
            }
        }

        Ok(())
    }

    fn aexp(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Id(..) | Token::Num(..) | Token::True | Token::False | Token::LeftParen => {
                    self.afac()?;
                    self.aexps()?;
                }
                _ => return Err(self.error("id, num, true, false, (")),
            }
        }

        Ok(())
    }

    fn aexps(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::End
                | Token::Then
                | Token::Else
                | Token::Do
                | Token::RightBracket
                | Token::Semicolon
                | Token::Or
                | Token::And
                | Token::LessThan
                | Token::Equals
                | Token::RightParen => (),
                Token::Plus => {
                    self.eat(Token::Plus)?;
                    self.afac()?;
                    self.aexps()?;
                }
                Token::Minus => {
                    self.eat(Token::Minus)?;
                    self.afac()?;
                    self.aexps()?;
                }
                _ => return Err(self.error("$, then, else, do, }}, ;, or, and, <, =, ), +, -")),
            }
        }

        Ok(())
    }

    fn afac(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Id(..) | Token::Num(..) | Token::True | Token::False | Token::LeftParen => {
                    self.atom()?;
                    self.afacs()?;
                }
                _ => return Err(self.error("id, num, true, false, (")),
            }
        }

        Ok(())
    }

    fn afacs(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::End
                | Token::Then
                | Token::Else
                | Token::Do
                | Token::RightBracket
                | Token::Semicolon
                | Token::Or
                | Token::And
                | Token::LessThan
                | Token::Equals
                | Token::Plus
                | Token::Minus
                | Token::RightParen => (),
                Token::Times => {
                    self.eat(Token::Times)?;
                    self.atom()?;
                    self.afacs()?;
                }
                _ => return Err(self.error("$, then, else, do, }}, ;, or, and, <, =, +, -, ), *")),
            }
        }

        Ok(())
    }

    fn atom(&mut self) -> Result<(), ParseError> {
        if let Some(tk) = self.peek() {
            match tk {
                Token::Id(s) => {
                    let name = s.clone();
                    self.eat(Token::Id(name))?;
                }
                Token::Num(s) => {
                    let name = s.clone();
                    self.eat(Token::Num(name))?;
                }
                Token::True => self.eat(Token::True)?,
                Token::False => self.eat(Token::False)?,
                Token::LeftParen => {
                    self.eat(Token::LeftParen)?;
                    self.bexp()?;
                    self.eat(Token::RightParen)?;
                }
                _ => return Err(self.error("id, num, true, false, (")),
            }
        }

        Ok(())
    }
}
