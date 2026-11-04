# while_interpreter

An interpreter for **While**, a small imperative language with integer variables, assignments, conditionals and loops. It is written in Rust with no external dependencies and has three stages:

1. **Lexer**: turns the source text into tokens.
2. **Parser**: a hand-written LL(1) recursive-descent parser that builds an abstract syntax tree.
3. **Evaluator**: walks the tree and updates a map from variable names to `i64` values.

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

```console
$ cargo run -- factorial.wh 5
Terminal state:
{"x": 120, "a": 0}
```

## Usage

```
cargo run -- <file> [while_arg] [--print-tokens] [--print-ast]
```

| Argument | Meaning |
| --- | --- |
| `<file>` | Path to a While source file (by convention `*.wh`). |
| `while_arg` | Optional integer. It becomes the starting value of variable `a`. |
| `--print-tokens` | Print the token list before parsing. |
| `--print-ast` | Print the parsed syntax tree before running. |

When the program finishes, the interpreter prints the final value of every variable. If lexing or parsing fails, it prints the error instead.

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
- **Variables don't need declaring.** A variable that has never been assigned reads as `0`.
- **Numbers and booleans can stand in for each other.** In a condition, a number counts as true when it is greater than `0`. In arithmetic, `true` counts as `1` and `false` as `0`, so `x <- (a = 3) * 10` sets `x` to `10` when `a` is 3.
- **Arithmetic uses 64-bit signed integers.** Overflow panics in debug builds.

## Development

```sh
cargo build
cargo test                                   # all tests
cargo test lexer::                           # one module
cargo test parser::tests::boolean_precedence # one test
cargo clippy
cargo fmt
```

Each module has unit tests at the bottom of its file:

| File | Contents |
| --- | --- |
| `src/main.rs` | Command-line parsing and the lexer → parser → evaluator pipeline. |
| `src/lexer.rs` | `Lexer` and `Token`. |
| `src/parser.rs` | `Parser` and the `Stmt` / `Exp` syntax tree. |
| `src/eval.rs` | `Evaluator`, plus whole-program tests (including `factorial.wh`). |
