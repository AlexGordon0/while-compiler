# while_compiler

A compiler and interpreter for **While**, a small imperative language with integer variables, assignments, conditionals and loops. It is written in Rust with no external dependencies.

```
source → Lexer → Parser → syntax tree ─┬─ Evaluator  → runs the program directly
                                       └─ CodeGen    → x86-64 NASM assembly (--emit-asm)
```

1. **Lexer**: turns the source text into tokens.
2. **Parser**: a hand-written LL(1) recursive-descent parser that builds an abstract syntax tree.
3. Then one of two back ends:
   - **Evaluator**: walks the tree and updates a map from variable names to `i64` values.
   - **Code generator**: turns the tree into x86-64 assembly for Linux. You assemble it with NASM and link it against libc with gcc.

## Example

`factorial.wh` computes the factorial of `a` and stores it in `x`:

```
x <- 1;
while 1 < a do {
    x <- x * a;
    a <- a - 1
};
a <- 0
```

Running it with the interpreter:

```console
$ cargo run -- factorial.wh 5
Terminal state:
{"x": 120, "a": 0}
```

Compiling it to a native executable (needs `nasm` and `gcc`):

```console
$ cargo run -q -- --emit-asm factorial.wh > factorial.asm
$ nasm -f elf64 factorial.asm -o factorial.o
$ gcc -no-pie factorial.o -o factorial
$ ./factorial 5
a: 0
x: 120
```

## Usage

```
cargo run -- <file> [while_arg] [--print-tokens] [--print-ast] [--emit-asm]
```

| Argument | Meaning |
| --- | --- |
| `<file>` | Path to a While source file (by convention `*.wh`). |
| `while_arg` | Optional integer that becomes the starting value of variable `a`. It only applies when interpreting. With `--emit-asm` it is ignored; pass it to the compiled program instead. |
| `--print-tokens` | Print the token list before parsing. |
| `--print-ast` | Print the parsed syntax tree. |
| `--emit-asm` | Print x86-64 NASM assembly to stdout instead of running the program. `--print-tokens` and `--print-ast` output then goes to stderr, so it doesn't end up in the assembly. |

When interpreting, the final value of every variable is printed when the program finishes. A compiled program does the same: it prints one `name: value` line per variable, sorted by name, and takes `a` as its optional first command-line argument.

If lexing or parsing fails, the error is printed and the exit status is 1.

## The language

### Grammar

`$` marks the end of input and `ε` the empty string.

```
Prog   -> Stmt Stmts $

Stmt   -> if BExp then Stmt else Stmt
       |  while BExp do Stmt
       |  skip
       |  id <- AExp
       |  {Stmt Stmts}

Stmts  -> ; Stmt Stmts
       |  ε

BExp   -> BFac BExps

BExps  -> || BFac BExps
       |  ε

BFac   -> BNeg BFacs

BFacs  -> && BNeg BFacs
       |  ε

BNeg   -> ! BNeg
       |  BRel

BRel   -> AExp BRels

BRels  -> < AExp
       |  = AExp
       |  ε

AExp   -> AFac AExps

AExps  -> + AFac AExps
       |  - AFac AExps
       |  ε

AFac   -> Atom AFacs

AFacs  -> * Atom AFacs
       |  ε

Atom   -> num
       |  true
       |  false
       |  id
       |  ( BExp )
```

Each nonterminal has a function of the same name in `src/parser.rs`.

### Tokens

| Token | Form |
| --- | --- |
| `id` | A lowercase letter followed by letters, digits or `_` (e.g. `x`, `count_2`). Keywords are reserved. |
| `num` | A non-negative decimal integer that fits in an `i64`. Negative values can be written as `0 - n`. |
| Keywords | `if` `then` `else` `while` `do` `skip` `true` `false` |
| Operators | `<-` `+` `-` `*` `<` `=` `!` `&&` `\|\|` |
| Punctuation | `(` `)` `{` `}` `;` |

Whitespace is ignored. The language has no comments.

### Things to know

- **`;` separates statements; it does not end them.** `x <- 1; y <- 2` is valid, but `x <- 1;` is a parse error.
- **Precedence**, from loosest to tightest: `||`, `&&`, `!`, `<` / `=`, `+` / `-`, `*`. Use parentheses to override it. Comparisons don't chain, so `1 < 2 < 3` is a parse error.
- **Every `if` needs an `else`.** Use `skip` for a branch that does nothing: `if x < 0 then x <- 0 else skip`.
- **Variables don't need declaring.** A variable that has never been assigned reads as `0`, and it isn't included in the printed output.
- **Numbers and booleans can stand in for each other.** In a condition, a number counts as true when it is greater than `0`. In arithmetic, `true` counts as `1` and `false` as `0`, so `x <- (a = 3) * 10` sets `x` to `10` when `a` is 3.
- **Arithmetic uses 64-bit signed integers.** The interpreter and compiled programs behave the same except in two cases:

  | Case | Interpreter | Compiled program |
  | --- | --- | --- |
  | Overflow | Panics in debug builds | Wraps around |
  | Non-integer argument | Rejects it | Reads it as `0` (it uses `atol`) |

## How the code generator works

- Every expression leaves its value in `rax`. Booleans are `0`/`1`, and a condition is true when `rax > 0`. These are the same rules the interpreter uses, so the two back ends agree.
- Binary operators evaluate the left operand, push it, evaluate the right operand into `rcx`, pop the left operand back into `rax`, and combine them. `&&` and `||` short-circuit.
- Each variable `x` has an 8-byte slot `v_x` in `.bss` (zeroed at startup) and a 1-byte flag `set_x`, which records whether it was ever assigned. Only variables whose flag is set are printed, which matches the interpreter.
- Generated names are prefixed (`v_`, `set_`, `name_`, `L<n>_…`), so a While variable called `rax` or `section` can't clash with NASM keywords or with other labels.

## Development

```sh
cargo build
cargo test                                   # all tests
cargo test codegen::                         # one module
cargo test parser::tests::boolean_precedence # one test
cargo clippy
cargo fmt
```

Each module has unit tests at the bottom of its file:

| File | Contents |
| --- | --- |
| `src/main.rs` | Command-line parsing and the lexer → parser → evaluator/code-generator pipeline. |
| `src/lexer.rs` | `Lexer` and `Token`. |
| `src/parser.rs` | `Parser` and the `Stmt` / `Exp` syntax tree. |
| `src/eval.rs` | `Evaluator`, plus whole-program tests (including `factorial.wh`). |
| `src/codegen.rs` | `CodeGen`. Its tests compile each program with `nasm` and `gcc`, run it, and check that it prints the same state as the interpreter. If either tool is missing, these tests skip themselves. |
