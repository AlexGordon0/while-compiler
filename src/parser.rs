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
            write!(
                f,
                "ParseError: Expected: [{}], Found: {token:?}",
                self.expected
            )
        } else {
            write!(f, "ParseError: Unexpected end of token input")
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    Skip,
    Assn(String, Exp),
    Seq(Box<Stmt>, Box<Stmt>),
    Cond(Exp, Box<Stmt>, Box<Stmt>),
    While(Exp, Box<Stmt>),
}

#[derive(Debug)]
pub enum Exp {
    True,
    False,
    Less(Box<Exp>, Box<Exp>),
    Eq(Box<Exp>, Box<Exp>),
    Not(Box<Exp>),
    And(Box<Exp>, Box<Exp>),
    Or(Box<Exp>, Box<Exp>),
    Var(String),
    Num(i64),
    Plus(Box<Exp>, Box<Exp>),
    Minus(Box<Exp>, Box<Exp>),
    Times(Box<Exp>, Box<Exp>),
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

    pub fn parse(&mut self) -> Result<Stmt, ParseError> {
        let s = self.prog()?;
        Ok(s)
    }

    fn prog(&mut self) -> Result<Stmt, ParseError> {
        let stmt = match self.peek() {
            Some(Token::If)
            | Some(Token::While)
            | Some(Token::Skip)
            | Some(Token::Id(..))
            | Some(Token::LeftBracket) => {
                let s = self.stmt()?;
                let s = self.stmts(s)?;
                self.eat(Token::End)?;
                s
            }
            _ => return Err(self.error("if, while, skip, id, (")),
        };
        Ok(stmt)
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError> {
        let token = self.peek().cloned();
        let stmt = match token {
            Some(Token::If) => {
                self.eat(Token::If)?;
                let e = self.bexp()?;
                self.eat(Token::Then)?;
                let s1 = self.stmt()?;
                self.eat(Token::Else)?;
                let s2 = self.stmt()?;
                Stmt::Cond(e, Box::new(s1), Box::new(s2))
            }
            Some(Token::While) => {
                self.eat(Token::While)?;
                let e = self.bexp()?;
                self.eat(Token::Do)?;
                let s = self.stmt()?;
                Stmt::While(e, Box::new(s))
            }
            Some(Token::Skip) => {
                self.eat(Token::Skip)?;
                Stmt::Skip
            }
            Some(Token::Id(s)) => {
                self.eat(Token::Id(s.clone()))?;
                self.eat(Token::Assn)?;
                let e = self.aexp()?;
                Stmt::Assn(s, e)
            }
            Some(Token::LeftBracket) => {
                self.eat(Token::LeftBracket)?;
                let s = self.stmt()?;
                let s = self.stmts(s)?;
                self.eat(Token::RightBracket)?;
                s
            }
            _ => return Err(self.error("if, while, skip, id, (")),
        };
        Ok(stmt)
    }

    fn stmts(&mut self, s: Stmt) -> Result<Stmt, ParseError> {
        let stmt = match self.peek() {
            Some(Token::End) | Some(Token::RightBracket) => s,
            Some(Token::Semicolon) => {
                self.eat(Token::Semicolon)?;
                let s1 = self.stmt()?;
                let s2 = self.stmts(s1)?;
                Stmt::Seq(Box::new(s), Box::new(s2))
            }
            _ => return Err(self.error("$, ), ;")),
        };

        Ok(stmt)
    }

    fn bexp(&mut self) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Id(..))
            | Some(Token::Not)
            | Some(Token::Num(..))
            | Some(Token::True)
            | Some(Token::False)
            | Some(Token::LeftParen) => {
                let e = self.bfac()?;
                self.bexps(e)?
            }
            _ => return Err(self.error("id, not, num, true, false, (")),
        };

        Ok(exp)
    }

    fn bexps(&mut self, e: Exp) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Then) | Some(Token::Do) | Some(Token::RightParen) => e,
            Some(Token::Or) => {
                self.eat(Token::Or)?;
                let e1 = self.bfac()?;
                let e2 = self.bexps(e1)?;
                Exp::Or(Box::new(e), Box::new(e2))
            }
            _ => return Err(self.error("then, do, ), or")),
        };

        Ok(exp)
    }

    fn bfac(&mut self) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Id(..))
            | Some(Token::Not)
            | Some(Token::Num(..))
            | Some(Token::True)
            | Some(Token::False)
            | Some(Token::LeftParen) => {
                let e = self.bneg()?;
                self.bfacs(e)?
            }
            _ => return Err(self.error("id, not, num, true, false, (")),
        };

        Ok(exp)
    }

    fn bfacs(&mut self, e: Exp) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Then) | Some(Token::Do) | Some(Token::Or) | Some(Token::RightParen) => e,
            Some(Token::And) => {
                self.eat(Token::And)?;
                let e1 = self.bneg()?;
                let e2 = self.bfacs(e1)?;
                Exp::And(Box::new(e), Box::new(e2))
            }
            _ => return Err(self.error("then, do, or, ), and")),
        };

        Ok(exp)
    }

    fn bneg(&mut self) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Id(..))
            | Some(Token::Num(..))
            | Some(Token::True)
            | Some(Token::False)
            | Some(Token::LeftParen) => self.brel()?,
            Some(Token::Not) => {
                self.eat(Token::Not)?;
                let e = self.bneg()?;
                Exp::Not(Box::new(e))
            }
            _ => return Err(self.error("id, num, true, false, (, not")),
        };

        Ok(exp)
    }

    fn brel(&mut self) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Id(..))
            | Some(Token::Num(..))
            | Some(Token::True)
            | Some(Token::False)
            | Some(Token::LeftParen) => {
                let e = self.aexp()?;
                self.brels(e)?
            }
            _ => return Err(self.error("id, num, true, false, (")),
        };

        Ok(exp)
    }

    fn brels(&mut self, e: Exp) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Then)
            | Some(Token::Do)
            | Some(Token::Or)
            | Some(Token::And)
            | Some(Token::RightParen) => e,
            Some(Token::LessThan) => {
                self.eat(Token::LessThan)?;
                let e1 = self.aexp()?;
                Exp::Less(Box::new(e), Box::new(e1))
            }
            Some(Token::Equals) => {
                self.eat(Token::Equals)?;
                let e1 = self.aexp()?;
                Exp::Eq(Box::new(e), Box::new(e1))
            }
            _ => return Err(self.error("then, do, or, and, ), <, =")),
        };

        Ok(exp)
    }

    fn aexp(&mut self) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Id(..))
            | Some(Token::Num(..))
            | Some(Token::True)
            | Some(Token::False)
            | Some(Token::LeftParen) => {
                let e = self.afac()?;
                self.aexps(e)?
            }
            _ => return Err(self.error("id, num, true, false, (")),
        };

        Ok(exp)
    }

    fn aexps(&mut self, e: Exp) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::End)
            | Some(Token::Then)
            | Some(Token::Else)
            | Some(Token::Do)
            | Some(Token::RightBracket)
            | Some(Token::Semicolon)
            | Some(Token::Or)
            | Some(Token::And)
            | Some(Token::LessThan)
            | Some(Token::Equals)
            | Some(Token::RightParen) => e,
            Some(Token::Plus) => {
                self.eat(Token::Plus)?;
                let e1 = self.afac()?;
                let e2 = self.aexps(e)?;
                Exp::Plus(Box::new(e2), Box::new(e1))
            }
            Some(Token::Minus) => {
                self.eat(Token::Minus)?;
                let e1 = self.afac()?;
                let e2 = self.aexps(e)?;
                Exp::Minus(Box::new(e2), Box::new(e1))
            }
            _ => return Err(self.error("$, then, else, do, }}, ;, or, and, <, =, ), +, -")),
        };

        Ok(exp)
    }

    fn afac(&mut self) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::Id(..))
            | Some(Token::Num(..))
            | Some(Token::True)
            | Some(Token::False)
            | Some(Token::LeftParen) => {
                let e = self.atom()?;
                self.afacs(e)?
            }
            _ => return Err(self.error("id, num, true, false, (")),
        };

        Ok(exp)
    }

    fn afacs(&mut self, e: Exp) -> Result<Exp, ParseError> {
        let exp = match self.peek() {
            Some(Token::End)
            | Some(Token::Then)
            | Some(Token::Else)
            | Some(Token::Do)
            | Some(Token::RightBracket)
            | Some(Token::Semicolon)
            | Some(Token::Or)
            | Some(Token::And)
            | Some(Token::LessThan)
            | Some(Token::Equals)
            | Some(Token::Plus)
            | Some(Token::Minus)
            | Some(Token::RightParen) => e,
            Some(Token::Times) => {
                self.eat(Token::Times)?;
                let e1 = self.atom()?;
                let e2 = self.afacs(e)?;
                Exp::Times(Box::new(e2), Box::new(e1))
            }
            _ => return Err(self.error("$, then, else, do, }}, ;, or, and, <, =, +, -, ), *")),
        };

        Ok(exp)
    }

    fn atom(&mut self) -> Result<Exp, ParseError> {
        let token = self.peek().cloned();
        let exp = match token {
            Some(Token::Id(s)) => {
                self.eat(Token::Id(s.clone()))?;
                Exp::Var(s)
            }
            Some(Token::Num(n)) => {
                self.eat(Token::Num(n))?;
                Exp::Num(n)
            }
            Some(Token::True) => {
                self.eat(Token::True)?;
                Exp::True
            }
            Some(Token::False) => {
                self.eat(Token::False)?;
                Exp::False
            }
            Some(Token::LeftParen) => {
                self.eat(Token::LeftParen)?;
                let e = self.bexp()?;
                self.eat(Token::RightParen)?;
                e
            }
            _ => return Err(self.error("id, num, true, false, (")),
        };

        Ok(exp)
    }
}
