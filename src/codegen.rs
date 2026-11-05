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
        code_gen.write("default rel");
        code_gen.write("global main");
        code_gen.write("extern printf, atol");

        // Prepare variables
        code_gen.vars.insert("a".to_string());
        code_gen.add_stmt_vars(&ast);

        let vars = code_gen.vars.clone();

        code_gen.write("section .data");
        code_gen.write("fmt: db \"%s: %ld\", 10, 0");

        for var in &vars {
            code_gen.write(&format!("name_{var}: db \"{var}\", 0"))
        }

        code_gen.write("section .bss");

        for var in &vars {
            code_gen.write(&format!("v_{var}: resq 1"));
            code_gen.write(&format!("set_{var}: resb 1"));
        }

        code_gen.write("section .text");
        code_gen.write("main:");
        code_gen.write("push rbp");
        code_gen.write("mov rbp, rsp");
        code_gen.write("cmp rdi, 2");
        code_gen.write("jl L_start");
        code_gen.write("mov rdi, [rsi + 8]");
        code_gen.write("call atol");
        code_gen.write("mov [v_a], rax");
        code_gen.write("mov byte [set_a], 1");

        code_gen.write("L_start:");
        code_gen.gen_stmt(&ast);

        // Print used variables
        for var in &vars {
            code_gen.write(&format!("cmp byte [set_{var}], 0"));
            code_gen.write(&format!("je L_skip_{var}"));
            code_gen.write("lea rdi, [fmt]");
            code_gen.write(&format!("lea rsi, [name_{var}]"));
            code_gen.write(&format!("mov rdx, [v_{var}]"));
            code_gen.write("xor eax, eax");
            code_gen.write("call printf");
            code_gen.write(&format!("L_skip_{var}:"));
        }

        code_gen.write("xor eax, eax");
        code_gen.write("pop rbp");
        code_gen.write("ret");

        code_gen.write("section .note.GNU-stack noalloc noexec nowrite progbits");

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
                self.write(&format!("mov [v_{x}], rax"));
                self.write(&format!("mov byte [set_{x}], 1"));
            }
            Stmt::Seq(s1, s2) => {
                self.gen_stmt(s1);
                self.gen_stmt(s2);
            }
            Stmt::Cond(c, s1, s2) => {
                let label = self.new_label();
                self.gen_exp(c);
                self.write("cmp rax, 0");
                self.write(&format!("jle {label}_else"));
                self.gen_stmt(s1);
                self.write(&format!("jmp {label}_end"));
                self.write(&format!("{label}_else:"));
                self.gen_stmt(s2);
                self.write(&format!("{label}_end:"));
            }
            Stmt::While(c, s) => {
                let label = self.new_label();
                self.write(&format!("{label}_top:"));
                self.gen_exp(c);
                self.write("cmp rax, 0");
                self.write(&format!("jle {label}_end"));
                self.gen_stmt(s);
                self.write(&format!("jmp {label}_top"));
                self.write(&format!("{label}_end:"));
            }
        }
    }

    fn gen_exp(&mut self, e: &Exp) {
        match e {
            Exp::True => self.write("mov rax, 1"),
            Exp::False => self.write("mov rax, 0"),
            Exp::Less(a, b) => {
                self.gen_exp(a);
                self.write("push rax");
                self.gen_exp(b);
                self.write("mov rcx, rax");
                self.write("pop rax");
                self.write("cmp rax, rcx");
                self.write("setl al");
                self.write("movzx rax, al");
            }
            Exp::Eq(a, b) => {
                self.gen_exp(a);
                self.write("push rax");
                self.gen_exp(b);
                self.write("mov rcx, rax");
                self.write("pop rax");
                self.write("cmp rax, rcx");
                self.write("sete al");
                self.write("movzx rax, al");
            }
            Exp::Not(a) => {
                self.gen_exp(a);
                self.write("cmp rax, 0");
                self.write("setle al");
                self.write("movzx rax, al");
            }
            Exp::And(a, b) => {
                let label = self.new_label();
                self.gen_exp(a);
                self.write("cmp rax, 0");
                self.write(&format!("jle {label}_f"));
                self.gen_exp(b);
                self.write("cmp rax, 0");
                self.write(&format!("jle {label}_f"));
                self.write("mov rax, 1");
                self.write(&format!("jmp {label}_e"));
                self.write(&format!("{label}_f: mov rax, 0"));
                self.write(&format!("{label}_e:"));
            }
            Exp::Or(a, b) => {
                let label = self.new_label();
                self.gen_exp(a);
                self.write("cmp rax, 0");
                self.write(&format!("jg {label}_t"));
                self.gen_exp(b);
                self.write("cmp rax, 0");
                self.write(&format!("jg {label}_t"));
                self.write("mov rax, 0");
                self.write(&format!("jmp {label}_e"));
                self.write(&format!("{label}_t: mov rax, 1"));
                self.write(&format!("{label}_e:"));
            }
            Exp::Var(s) => self.write(&format!("mov rax, [v_{s}]")),
            Exp::Num(n) => self.write(&format!("mov rax, {n}")),
            Exp::Plus(a, b) => {
                self.gen_exp(a);
                self.write("push rax");
                self.gen_exp(b);
                self.write("mov rcx, rax");
                self.write("pop rax");
                self.write("add rax, rcx");
            }
            Exp::Minus(a, b) => {
                self.gen_exp(a);
                self.write("push rax");
                self.gen_exp(b);
                self.write("mov rcx, rax");
                self.write("pop rax");
                self.write("sub rax, rcx");
            }
            Exp::Times(a, b) => {
                self.gen_exp(a);
                self.write("push rax");
                self.gen_exp(b);
                self.write("mov rcx, rax");
                self.write("pop rax");
                self.write("imul rax, rcx");
            }
        }
    }

    fn new_label(&mut self) -> String {
        let s = format!("L{}", self.labels);
        self.labels += 1;
        s
    }

    fn write(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push('\n');
    }
}
