use crate::parser::Exp;
use crate::parser::Stmt;
use std::collections::HashMap;

pub struct Evaluator<'a> {
    env: &'a mut HashMap<String, i64>,
}

impl<'a> Evaluator<'a> {
    pub fn new(state: &'a mut HashMap<String, i64>) -> Self {
        Self { env: state }
    }

    fn lookup_var(&self, var: &str) -> i64 {
        *self.env.get(var).unwrap_or(&0)
    }

    fn update_var(&mut self, var: &str, value: i64) {
        self.env.insert(var.to_string(), value);
    }

    pub fn execute_statement(&mut self, s: &Stmt) {
        match s {
            Stmt::Skip => (),
            Stmt::Assn(s, value) => self.update_var(s, self.evaluate_arithmetic(value)),
            Stmt::Seq(a, b) => {
                self.execute_statement(a);
                self.execute_statement(b);
            }
            Stmt::Cond(c, s1, s2) => {
                if self.evaluate_boolean(c) {
                    self.execute_statement(s1);
                } else {
                    self.execute_statement(s2);
                }
            }
            Stmt::While(c, s) => {
                while self.evaluate_boolean(c) {
                    self.execute_statement(s);
                }
            }
        }
    }

    fn evaluate_arithmetic(&self, e: &Exp) -> i64 {
        match e {
            Exp::Var(s) => self.lookup_var(s),
            Exp::Num(n) => *n,
            Exp::Plus(a, b) => self.evaluate_arithmetic(a) + self.evaluate_arithmetic(b),
            Exp::Minus(a, b) => self.evaluate_arithmetic(a) - self.evaluate_arithmetic(b),
            Exp::Times(a, b) => self.evaluate_arithmetic(a) * self.evaluate_arithmetic(b),
            _ => unreachable!(),
        }
    }

    fn evaluate_boolean(&self, b: &Exp) -> bool {
        match b {
            Exp::True => true,
            Exp::False => false,
            Exp::Less(a, b) => self.evaluate_arithmetic(a) < self.evaluate_arithmetic(b),
            Exp::Eq(a, b) => self.evaluate_arithmetic(a) == self.evaluate_arithmetic(b),
            Exp::Not(a) => !self.evaluate_boolean(a),
            Exp::And(a, b) => self.evaluate_boolean(a) && self.evaluate_boolean(b),
            Exp::Or(a, b) => self.evaluate_boolean(a) || self.evaluate_boolean(b),
            _ => self.evaluate_arithmetic(b) > 0,
        }
    }
}
