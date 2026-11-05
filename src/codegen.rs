use crate::parser::Exp;
use crate::parser::Stmt;
use std::collections::BTreeSet;

pub struct CodeGen {
    out: String,
    labels: usize,
    vars: BTreeSet<String>,
}

impl CodeGen {
    pub fn generate(ast: &Stmt) -> String {
        let mut code_gen = CodeGen {
            out: String::new(),
            labels: 0,
            vars: BTreeSet::new(),
        };

        // Add header
        code_gen.raw("default rel");
        code_gen.raw("global main");
        code_gen.raw("extern printf, atol");

        // Prepare variables
        code_gen.vars.insert("a".to_string());
        code_gen.add_stmt_vars(ast);

        let vars = code_gen.vars.clone();

        code_gen.section(".data");
        code_gen.raw("fmt: db \"%s: %ld\", 10, 0");

        for var in &vars {
            code_gen.raw(format!("name_{var}: db \"{var}\", 0"))
        }

        code_gen.section(".bss");

        for var in &vars {
            code_gen.raw(format!("v_{var}: resq 1"));
        }
        for var in &vars {
            code_gen.raw(format!("set_{var}: resb 1"));
        }

        code_gen.section(".text");
        code_gen.raw("main:");
        code_gen.instr("push rbp");
        code_gen.instr("mov rbp, rsp");
        code_gen.instr("cmp rdi, 2");
        code_gen.instr("jl L_start");
        code_gen.instr("mov rdi, [rsi + 8]");
        code_gen.instr("call atol");
        code_gen.instr("mov [v_a], rax");
        code_gen.instr("mov byte [set_a], 1");

        code_gen.label("L_start");
        code_gen.gen_stmt(ast);

        // Print used variables
        for var in &vars {
            code_gen.instr(format!("cmp byte [set_{var}], 0"));
            code_gen.instr(format!("je L_skip_{var}"));
            code_gen.instr("lea rdi, [fmt]");
            code_gen.instr(format!("lea rsi, [name_{var}]"));
            code_gen.instr(format!("mov rdx, [v_{var}]"));
            code_gen.instr("xor eax, eax");
            code_gen.instr("call printf");
            code_gen.label(format!("L_skip_{var}"));
        }

        code_gen.instr("xor eax, eax");
        code_gen.instr("pop rbp");
        code_gen.instr("ret");

        code_gen.section(".note.GNU-stack noalloc noexec nowrite progbits");

        code_gen.out
    }

    fn add_stmt_vars(&mut self, s: &Stmt) {
        match s {
            Stmt::Skip => (),
            Stmt::Assn(s, value) => {
                self.vars.insert(s.clone());
                self.add_exp_vars(value);
            }
            Stmt::Seq(s1, s2) => {
                self.add_stmt_vars(s1);
                self.add_stmt_vars(s2);
            }
            Stmt::Cond(c, s1, s2) => {
                self.add_exp_vars(c);
                self.add_stmt_vars(s1);
                self.add_stmt_vars(s2);
            }
            Stmt::While(c, s) => {
                self.add_exp_vars(c);
                self.add_stmt_vars(s);
            }
        }
    }

    fn add_exp_vars(&mut self, e: &Exp) {
        match e {
            Exp::Less(a, b)
            | Exp::Eq(a, b)
            | Exp::And(a, b)
            | Exp::Or(a, b)
            | Exp::Plus(a, b)
            | Exp::Minus(a, b)
            | Exp::Times(a, b) => {
                self.add_exp_vars(a);
                self.add_exp_vars(b);
            }
            Exp::Not(a) => self.add_exp_vars(a),
            Exp::Var(x) => {
                self.vars.insert(x.clone());
            }
            _ => (),
        }
    }

    fn gen_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Skip => (),
            Stmt::Assn(x, e) => {
                self.gen_exp(e);
                self.instr(format!("mov [v_{x}], rax"));
                self.instr(format!("mov byte [set_{x}], 1"));
            }
            Stmt::Seq(s1, s2) => {
                self.gen_stmt(s1);
                self.gen_stmt(s2);
            }
            Stmt::Cond(c, s1, s2) => {
                let label = self.new_label();
                self.gen_exp(c);
                self.instr("cmp rax, 0");
                self.instr(format!("jle {label}_else"));
                self.gen_stmt(s1);
                self.instr(format!("jmp {label}_end"));
                self.label(format!("{label}_else"));
                self.gen_stmt(s2);
                self.label(format!("{label}_end"));
            }
            Stmt::While(c, s) => {
                let label = self.new_label();
                self.label(format!("{label}_top"));
                self.gen_exp(c);
                self.instr("cmp rax, 0");
                self.instr(format!("jle {label}_end"));
                self.gen_stmt(s);
                self.instr(format!("jmp {label}_top"));
                self.label(format!("{label}_end"));
            }
        }
    }

    fn gen_exp(&mut self, e: &Exp) {
        match e {
            Exp::True => self.instr("mov rax, 1"),
            Exp::False => self.instr("mov rax, 0"),
            Exp::Less(a, b) => {
                self.gen_operands(a, b);
                self.instr("cmp rax, rcx");
                self.instr("setl al");
                self.instr("movzx rax, al");
            }
            Exp::Eq(a, b) => {
                self.gen_operands(a, b);
                self.instr("cmp rax, rcx");
                self.instr("sete al");
                self.instr("movzx rax, al");
            }
            Exp::Plus(a, b) => {
                self.gen_operands(a, b);
                self.instr("add rax, rcx");
            }
            Exp::Minus(a, b) => {
                self.gen_operands(a, b);
                self.instr("sub rax, rcx");
            }
            Exp::Times(a, b) => {
                self.gen_operands(a, b);
                self.instr("imul rax, rcx");
            }
            Exp::Not(a) => {
                self.gen_exp(a);
                self.instr("cmp rax, 0");
                self.instr("setle al");
                self.instr("movzx rax, al");
            }
            Exp::And(a, b) => {
                let label = self.new_label();
                self.gen_exp(a);
                self.instr("cmp rax, 0");
                self.instr(format!("jle {label}_f"));
                self.gen_exp(b);
                self.instr("cmp rax, 0");
                self.instr(format!("jle {label}_f"));
                self.instr("mov rax, 1");
                self.instr(format!("jmp {label}_e"));
                self.label(format!("{label}_f"));
                self.instr("mov rax, 0");
                self.label(format!("{label}_e"));
            }
            Exp::Or(a, b) => {
                let label = self.new_label();
                self.gen_exp(a);
                self.instr("cmp rax, 0");
                self.instr(format!("jg {label}_t"));
                self.gen_exp(b);
                self.instr("cmp rax, 0");
                self.instr(format!("jg {label}_t"));
                self.instr("mov rax, 0");
                self.instr(format!("jmp {label}_e"));
                self.label(format!("{label}_t"));
                self.instr("mov rax, 1");
                self.label(format!("{label}_e"));
            }
            Exp::Var(s) => self.instr(format!("mov rax, [v_{s}]")),
            Exp::Num(n) => self.instr(format!("mov rax, {n}")),
        }
    }

    fn gen_operands(&mut self, a: &Exp, b: &Exp) {
        self.gen_exp(a);
        self.instr("push rax");
        self.gen_exp(b);
        self.instr("mov rcx, rax");
        self.instr("pop rax");
    }

    fn new_label(&mut self) -> String {
        let s = format!("L{}", self.labels);
        self.labels += 1;
        s
    }

    fn instr(&mut self, line: impl AsRef<str>) {
        self.out.push_str("    ");
        self.out.push_str(line.as_ref());
        self.out.push('\n');
    }

    fn label(&mut self, name: impl AsRef<str>) {
        self.out.push_str(name.as_ref());
        self.out.push_str(":\n");
    }

    fn section(&mut self, name: &str) {
        self.out.push_str("\nsection ");
        self.out.push_str(name);
        self.out.push('\n');
    }

    fn raw(&mut self, line: impl AsRef<str>) {
        self.out.push_str(line.as_ref());
        self.out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::collections::HashMap;
    use std::fs;
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static BUILD_ID: AtomicUsize = AtomicUsize::new(0);

    fn parse(input: &str) -> Stmt {
        let tokens = Lexer::new(input).lex().unwrap();
        Parser::new(&tokens).parse().unwrap()
    }

    fn toolchain_available() -> bool {
        let found = |tool: &str| Command::new(tool).arg("--version").output().is_ok();
        found("nasm") && found("gcc")
    }

    fn check(cmd: &mut Command) {
        let output = cmd.output().unwrap();
        assert!(
            output.status.success(),
            "{cmd:?} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Compiles `input` with nasm and gcc, runs it with `arg`, and returns the variables it prints.
    fn compile_and_run(input: &str, arg: Option<i64>) -> HashMap<String, i64> {
        let id = BUILD_ID.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("while_codegen_{}_{id}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let (asm, obj, exe) = (dir.join("prog.asm"), dir.join("prog.o"), dir.join("prog"));

        fs::write(&asm, CodeGen::generate(&parse(input))).unwrap();
        check(
            Command::new("nasm")
                .args(["-f", "elf64"])
                .arg(&asm)
                .arg("-o")
                .arg(&obj),
        );
        check(
            Command::new("gcc")
                .arg("-no-pie")
                .arg(&obj)
                .arg("-o")
                .arg(&exe),
        );

        let mut run = Command::new(&exe);
        if let Some(a) = arg {
            run.arg(a.to_string());
        }
        let output = run.output().unwrap();
        assert!(
            output.status.success(),
            "program exited with {}",
            output.status
        );
        fs::remove_dir_all(&dir).ok();

        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| {
                let (name, value) = line.split_once(": ").unwrap();
                (name.to_string(), value.parse().unwrap())
            })
            .collect()
    }

    fn interpret(input: &str, arg: Option<i64>) -> HashMap<String, i64> {
        let mut state = HashMap::new();
        if let Some(a) = arg {
            state.insert("a".to_string(), a);
        }
        Evaluator::new(&mut state).execute_statement(&parse(input));
        state
    }

    // Asserts that the compiled program prints exactly the interpreter's terminal state.
    fn assert_matches_interpreter(input: &str, arg: Option<i64>) {
        if !toolchain_available() {
            eprintln!("skipping: nasm or gcc not found");
            return;
        }
        assert_eq!(
            compile_and_run(input, arg),
            interpret(input, arg),
            "program: {input}, arg: {arg:?}"
        );
    }

    #[test]
    fn skip_without_argument_prints_nothing() {
        assert_matches_interpreter("skip", None);
    }

    #[test]
    fn argument_is_stored_in_a() {
        assert_matches_interpreter("skip", Some(7));
        assert_matches_interpreter("skip", Some(-5));
        assert_matches_interpreter("skip", Some(0));
        assert_matches_interpreter("x <- a * 2", Some(21));
    }

    #[test]
    fn assignment() {
        assert_matches_interpreter("x <- 5", None);
        assert_matches_interpreter("x <- 1; x <- x + 1; x <- x * 10", None);
    }

    #[test]
    fn unset_variables_read_as_zero_and_are_not_printed() {
        assert_matches_interpreter("x <- y + 1", None);
    }

    #[test]
    fn large_literal() {
        assert_matches_interpreter("x <- 9223372036854775807", None);
    }

    #[test]
    fn arithmetic() {
        for prog in [
            "x <- 2 * 3 + 4 * 5",
            "x <- (2 + 3) * 4",
            "x <- 10 - 3 - 2",
            "x <- 20 - 2 * 3 - 4",
            "x <- 1 - 2 + 3",
            "x <- (10 - 3) - (4 - 1)",
            "x <- 0 - 5",
            "x <- (0 - 3) * (0 - 4)",
        ] {
            assert_matches_interpreter(prog, None);
        }
    }

    #[test]
    fn boolean_operators_in_conditions() {
        let conditions = [
            "true",
            "false",
            "!true",
            "!!true",
            "1 = 1",
            "1 = 2",
            "1 < 2",
            "2 < 1",
            "0 - 1 < 0",
            "true && false",
            "true && true",
            "false || true",
            "false || false",
            "true || false && false",
            "(true || false) && false",
            "!(1 = 1) || 1 < 2 && 2 < 3",
        ];
        // One program with every condition, which also checks labels stay unique.
        let prog = conditions
            .iter()
            .enumerate()
            .map(|(i, c)| format!("if {c} then r{i} <- 1 else r{i} <- 0"))
            .collect::<Vec<_>>()
            .join("; ");
        assert_matches_interpreter(&prog, None);
    }

    #[test]
    fn boolean_as_arithmetic_is_one_or_zero() {
        for prog in [
            "x <- (1 < 2)",
            "x <- (2 < 1)",
            "x <- true",
            "x <- false",
            "x <- (!(1 = 1) || true && 1 < 2)",
            "x <- (a = 3) * 10 + 1",
        ] {
            assert_matches_interpreter(prog, Some(3));
        }
    }

    #[test]
    fn arithmetic_as_condition_is_truthy_when_positive() {
        let prog = "if a then r <- 1 else r <- 0";
        for a in [1, 0, -1, 100] {
            assert_matches_interpreter(prog, Some(a));
        }
        assert_matches_interpreter("if !a then r <- 1 else r <- 0", Some(-1));
        assert_matches_interpreter("if a && 2 then r <- 1 else r <- 0", Some(5));
        assert_matches_interpreter("if a || 0 then r <- 1 else r <- 0", Some(0));
    }

    #[test]
    fn conditional_branches() {
        let prog = "if a < 5 then r <- 1 else r <- 2";
        assert_matches_interpreter(prog, Some(3));
        assert_matches_interpreter(prog, Some(5));
    }

    #[test]
    fn variable_assigned_only_in_untaken_branch_is_not_printed() {
        assert_matches_interpreter("if false then x <- 1 else skip", None);
        assert_matches_interpreter("while false do x <- 1", None);
    }

    #[test]
    fn nested_conditionals() {
        let prog = "if a < 10 then { if a < 5 then r <- 1 else r <- 2 } else { if a < 15 then r <- 3 else r <- 4 }";
        for a in [0, 7, 12, 20] {
            assert_matches_interpreter(prog, Some(a));
        }
    }

    #[test]
    fn while_loops() {
        assert_matches_interpreter(
            "i <- 0; s <- 0; while i < 5 do { i <- i + 1; s <- s + i }",
            None,
        );
        assert_matches_interpreter(
            "i <- 0; n <- 0; while i < 3 do { j <- 0; while j < 4 do { n <- n + 1; j <- j + 1 }; i <- i + 1 }",
            None,
        );
    }

    #[test]
    fn factorial_example() {
        let prog = include_str!("../factorial.wh");
        for a in [0, 1, 5, 10, 20] {
            assert_matches_interpreter(prog, Some(a));
        }
        assert_matches_interpreter(prog, None);
    }

    #[test]
    fn variables_named_like_asm_keywords() {
        assert_matches_interpreter(
            "rax <- 1; byte <- 2; section <- 3; main <- 4; printf <- 5; fmt <- 6; start <- 7; ret <- rax + byte",
            None,
        );
    }

    #[test]
    fn variables_named_like_generated_labels() {
        assert_matches_interpreter(
            "l0 <- 1; skip_a <- 2; set_a <- 3; v_a <- 4; name_a <- 5; a <- 6",
            None,
        );
    }

    #[test]
    fn overflow_wraps() {
        // The interpreter panics on overflow in debug builds; compiled code wraps.
        if !toolchain_available() {
            return;
        }
        let state = compile_and_run("x <- 9223372036854775807 + 1", None);
        assert_eq!(state["x"], i64::MIN);
    }

    #[test]
    fn labels_are_unique() {
        let asm = CodeGen::generate(&parse(
            "while a < 3 do { if a = 1 && b < 2 || c then a <- a + 1 else { while b < 2 do b <- b + 1 } }; if a then skip else skip",
        ));
        let mut seen = BTreeSet::new();
        for line in asm.lines() {
            let line = line.trim();
            if let Some((label, _)) = line.split_once(':')
                && !label.contains(' ')
            {
                assert!(seen.insert(label.to_string()), "duplicate label {label}");
            }
        }
    }
}
