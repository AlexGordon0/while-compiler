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
            _ => self.evaluate_boolean(e) as i64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn run_with(input: &str, initial: &[(&str, i64)]) -> HashMap<String, i64> {
        let tokens = Lexer::new(input).lex().unwrap();
        let ast = Parser::new(&tokens).parse().unwrap();
        let mut state: HashMap<String, i64> =
            initial.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        Evaluator::new(&mut state).execute_statement(&ast);
        state
    }

    fn run(input: &str) -> HashMap<String, i64> {
        run_with(input, &[])
    }

    #[test]
    fn skip_leaves_state_unchanged() {
        assert!(run("skip").is_empty());
        assert_eq!(run_with("skip", &[("a", 7)])["a"], 7);
    }

    #[test]
    fn assignment() {
        assert_eq!(run("x <- 5")["x"], 5);
    }

    #[test]
    fn unset_variables_read_as_zero() {
        assert_eq!(run("x <- y + 1")["x"], 1);
    }

    #[test]
    fn arithmetic_precedence() {
        assert_eq!(run("x <- 2 * 3 + 4 * 5")["x"], 26);
        assert_eq!(run("x <- (2 + 3) * 4")["x"], 20);
    }

    #[test]
    fn subtraction_is_left_associative() {
        assert_eq!(run("x <- 10 - 3 - 2")["x"], 5);
        assert_eq!(run("x <- 20 - 2 * 3 - 4")["x"], 10);
        assert_eq!(run("x <- 1 - 2 + 3")["x"], 2);
        assert_eq!(run("x <- (10 - 3) - (4 - 1)")["x"], 4);
    }

    #[test]
    fn negative_results() {
        assert_eq!(run("x <- 0 - 5")["x"], -5);
    }

    #[test]
    fn sequence_runs_in_order() {
        assert_eq!(run("x <- 1; x <- x + 1; x <- x * 10")["x"], 20);
    }

    #[test]
    fn conditional_branches() {
        let prog = "if a < 5 then r <- 1 else r <- 2";
        assert_eq!(run_with(prog, &[("a", 3)])["r"], 1);
        assert_eq!(run_with(prog, &[("a", 5)])["r"], 2);
    }

    #[test]
    fn boolean_operators() {
        let check = |cond: &str, expected: i64| {
            let prog = format!("if {cond} then r <- 1 else r <- 0");
            assert_eq!(run(&prog)["r"], expected, "{cond}");
        };
        check("true", 1);
        check("false", 0);
        check("!true", 0);
        check("!!true", 1);
        check("1 = 1", 1);
        check("1 = 2", 0);
        check("1 < 2", 1);
        check("2 < 1", 0);
        check("true && false", 0);
        check("true && true", 1);
        check("false || true", 1);
        check("false || false", 0);
        check("true || false && false", 1);
        check("(true || false) && false", 0);
    }

    #[test]
    fn arithmetic_as_condition_is_truthy_when_positive() {
        let prog = "if a then r <- 1 else r <- 0";
        assert_eq!(run_with(prog, &[("a", 1)])["r"], 1);
        assert_eq!(run_with(prog, &[("a", 0)])["r"], 0);
        assert_eq!(run_with(prog, &[("a", -1)])["r"], 0);
    }

    #[test]
    fn boolean_as_arithmetic_is_one_or_zero() {
        assert_eq!(run("x <- (1 < 2)")["x"], 1);
        assert_eq!(run("x <- (2 < 1)")["x"], 0);
        assert_eq!(run("x <- true")["x"], 1);
        assert_eq!(run("x <- false")["x"], 0);
        assert_eq!(run("x <- (!(1 = 1) || true && 1 < 2)")["x"], 1);
        assert_eq!(run_with("x <- (a = 3) * 10 + 1", &[("a", 3)])["x"], 11);
        assert_eq!(run_with("x <- (a = 3) * 10 + 1", &[("a", 4)])["x"], 1);
    }

    #[test]
    fn while_loop_counts() {
        let state = run("i <- 0; s <- 0; while i < 5 do { i <- i + 1; s <- s + i }");
        assert_eq!(state["i"], 5);
        assert_eq!(state["s"], 15);
    }

    #[test]
    fn while_loop_with_false_condition_never_runs() {
        assert!(!run("while false do x <- 1").contains_key("x"));
    }

    #[test]
    fn nested_loops() {
        let prog = "i <- 0; n <- 0; while i < 3 do { j <- 0; while j < 4 do { n <- n + 1; j <- j + 1 }; i <- i + 1 }";
        assert_eq!(run(prog)["n"], 12);
    }

    #[test]
    fn factorial_example() {
        let prog = include_str!("../factorial.wh");
        for (a, expected) in [(0, 1), (1, 1), (5, 120), (10, 3628800)] {
            let state = run_with(prog, &[("a", a)]);
            assert_eq!(state["x"], expected, "factorial of {a}");
            assert_eq!(state["a"], 0);
        }
    }
}
