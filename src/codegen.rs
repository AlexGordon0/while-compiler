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
