# Text mode: root cause and spec (G8/G9)

Status: analysis only, at c846b58. No fix code is written yet.

## 1. Root cause

`\text{…}` bodies never get a text-mode lexer. They go through three passes,
and each pass was written for math mode:

1. `preprocess` (src/parser/preproc.rs:23-43) rewrites the whole input string,
   text bodies included.
2. `tokenize` (src/parser/token.rs:97-117) lexes the whole string with **math**
   rules, text bodies included.
3. `collect_group_text` (src/parser/parse.rs:515-563) turns the math tokens
   back into a `String`. This step loses information.

The printer (`push_escaped`, src/parser/ast.rs:2140-2154) then re-escapes
that string. It uses a fourth escaping rule that matches none of the others.
The AST stores decoded characters (`MathNode::Text(String, _)`,
ast.rs:591), so it cannot tell `\)` from `)`, `\\` from `\`, `\{` from `{`,
or a nested `\text{` from the literal characters `\text{`. Then:

* Any math-level code that looks at raw tokens *across* a text group sees
  different tokens in the input and in the reprint. G9 hits this.
* Any string rewrite that runs before lexing can match bytes that the printer
  produced inside a text body. G8 hits this.
* Any lossy decode makes `parse(print(parse(s)))` differ from `parse(s)`, or
  makes `parse(s)` silently wrong.

Each fuzz group so far (G1-G9) patched one of these meeting points. The bugs
keep coming back because text is not opaque to the math passes, and its
encoding is not one round-trippable bijection.

### 1a. Pre-parse string rewrites (preproc.rs)

None of these knows about text mode. The printer reverses none of them.

| # | Rewrite | Line | Escape-aware? | Applies inside `\text`? | Printer reverses? |
|---|---|---|---|---|---|
| P1 | `trim_math`: trim the outer whitespace, keep the space of a trailing odd `\ ` | 26, 54-69 | parity of backslashes | n/a (outer only) | n/a; reprint does not end in a bare space |
| P2 | `{ (}` → `(` | 32 | yes (`replace_unescaped`, 72-85) | **yes** | no |
| P3 | `{ )}` → `)` | 33 | yes | **yes**: `\text{{ )}}` gives `Text(")")` | no |
| P4 | `{ ( }` → `(` | 34 | yes | yes | no |
| P5 | `{ ) }` → `)` | 35 | yes | yes | no |
| P6 | `{[}` → `[` | 36 | yes | yes | no |
| P7 | `{]}` → `]` | 37 | yes | yes | no |
| P8 | `{\rm x}` etc. → `\mathrm{x}` (`convert_plain_tex_font_scopes`) | 39, 96-127 | **no** (`find`, first `}` after it, matches `\rmfoo`) | **yes**: `\text{{\rm a}}` gives `Text("\mathrm{a}")` | no |
| P9 | `{a \over b}` → `\frac{a}{b}` | 40, 134 ff. | partly (checks the control word, scans the layers) | **yes**: `\text{{a \over b}}` gives `Text("\frac{a}{b}")` | no |
| P10 | `\choose` → `\binom` | 41, 139 | same as P9 | yes | no |
| P11 | `\mbox{` → `\text{` | 42 | **no** (plain `str::replace`) | yes: `\text{\mbox{x}}` gives `Text("\text{x}")` | no |

### 1b. Math lexer applied to text (token.rs)

| # | Behaviour | Line | Effect in text |
|---|---|---|---|
| L1 | `%` comment skipped | 103 | `\text{a%x⏎b}` gives `"ab"` (TeX would agree, but the printer escapes `%`, so it is fine) |
| L2 | `\t \n \r VT FF` dropped | 105 | `\text{a⇥b}` gives `"ab"`. In TeX these are spaces. |
| L3 | spaces after a control word eaten | 148-150 | `\text{a\foo b}` gives `"a\foob"`, so a word is merged |
| L4 | one-char control symbol becomes `Command(c)` | 152-154 | input for D1 |
| L5 | `^ _ & $ $$` become structural tokens | 107-115 | `collect_group_text` rejects them ("unexpected token … in group text"). But `\text{$x$}` is legal LaTeX. |

### 1c. Decoding in collect_group_text (parse.rs:515-563)

| # | Input token | Stored as | Loses |
|---|---|---|---|
| D1 | `Command(c)`, c one char, not whitespace (`\{ \} \\ \) \( \$ \% \& \# \_ \,` …) | the bare char `c` | escape vs literal; `\\` (line break) becomes the char `\`; `\,` (thin space) becomes `,`; `\)` becomes `)` |
| D2 | `Command(c)`, c whitespace (control space) | `' '` | control space vs ordinary space |
| D3 | `Command(name)`, multi-letter | `'\'` + name, **without a terminator** | the nested `\text{…}` is not parsed. Its braces become D4 chars. The space after the name is gone (L3). |
| D4 | `BeginGroup`/`EndGroup` (inner) | the chars `{` / `}` | brace group vs escaped brace (`{` and `\{` give the same string) |
| D5 | `Space` | `' '` | runs are kept as written |

### 1d. Printer (ast.rs:1700-1717, 2140-2154)

* Escapes `\ { } % $ & _ ^` everywhere, and `' ' [ ]` at index 0 only.
* Does **not** escape `#` or `~`, which are plain chars after D1. Harmless
  in this lexer, but not TeX.
* Prints `)` and `(` bare. They are the only text chars that math-level
  lookahead also treats as special. **G9 is caused here.**
* Non-alphabetic text under `\mathrm` and similar is wrapped in
  `\mathrm{\text{…}}` (1708-1716). That is a second text-style encoding.
* `collapse_text` (parse.rs:1114-1133) merges adjacent `Text` nodes of the
  same style. That is fine only while print and parse stay inverse.

### 1e. Places where escapes are handled twice or inconsistently

* **Backslash parity** is computed three times, in three ways:
  `ends_with_odd_backslash` (preproc.rs:87-94) for P2-P7 and P1, nothing for
  P8 and P11, and the tokenizer's left-to-right `command()` (token.rs:139-156).
  These three disagree whenever a rewrite changes the text before a later one
  runs.
* **Control space** has its own case in P1 (trim), in L4/D2 (decoded to `' '`),
  and in the printer (prints `\ ` only at index 0, otherwise a bare space,
  which is legal because the tokenizer keeps `Space` in groups). So the
  meaning of `\ ` depends on its position after the reprint.
* **Braces**: `{` inside text can be a group (D4) or an escape (D1). Both give
  the same char. The printer always emits `\{`. So `\text{{}}` reprints as
  `\text{\{\}}`, a different token stream. It decodes to the same string, but
  stream-level code (lookahead, preprocess) sees something different.
* **Paren lookahead** `script_follows_parens` (parse.rs:396-425, called from
  parse/drive.rs:497 and 728) scans the raw token stream. It counts `)` at
  any brace depth, including inside `\text{…}` and ordinary groups.

### 1f. G8 in these terms

Input `\text{\{\ )}`. D1 and D2 give `Text("{ )")`. The printer gives
`\text {\{ )}`. Before 911515a, P3 matched `{ )}` starting at the brace of
`\{`, and the reprint lost its closing `}`. 911515a made P2-P7 parity-aware,
but P2-P7 still run inside text (P3 row above). P8 and P11 are still
parity-blind.

### 1g. G9 in these terms

Input (base64 `KD8/…IA==`):

```
(???????\text {\text {\ }\)') t{Lambda}Bb) }\ 
```

Minimized by deletion: `(\text{{}\)'{}}`.

* Parse 1. The `(` lookahead (parse.rs:396) scans the tokens. The `)` is
  `Command(")")`, not `Char(')')`, so no match, and `(` is an ordinary atom.
  `\text` body: the inner `\text` becomes the literal `\text` (D3), braces
  become chars (D4), `\ ` becomes `' '` (D2), `\)` becomes `)` (D1). The AST is:

  ```
  Row([Atom('(',Open), Atom('?',Punct)×7,
       Text("\\text{ })') t{Lambda}Bb) ", Text), Space(ControlSpace)])
  ```

* Reprint (actual):

  ```
  {(}{?}{?}{?}{?}{?}{?}{?}{\text {\\text\{ \})') t\{Lambda\}Bb) }}{\ }
  ```

* Parse 2. The `)` inside the text body is now `Char(')')` followed by
  `Char('\'')`. `script_follows_parens` returns true, and the parser enters
  paren-group mode at `(`. Paren mode parses atoms, one of which consumes the
  whole `{\text{…}}` group. It never finds a `)` at its own level, and it hits
  the `}` that closes the `{(}` wrapper. The result is
  `Malformed("unexpected token }")`. Minimal form: `(\text{)'}` gives
  "unmatched '('". The same defect fires with no text at all: `({a)'}` and
  `(\mathrm{a)'})` are rejected today.

The root cause has two halves. The text AST forgets that `)` was escaped
(D1), and the math lookahead does not treat a brace group as opaque (1e).
Either half alone is a bug.

## 2. Text-mode spec

### 2.1 Grammar (after the `\text` / `\mbox` / `\textrm`… command, in its argument)

```
text_arg   ::= ws* "{" text_body "}"
text_body  ::= text_item*
text_item  ::= char | space | escape | ctrl_space | group | nested | math_cmd
char       ::= any Unicode scalar except  \ { } $ % & # ^ _ ~ and whitespace
space      ::= (" " | "\t" | "\n" | "\r" | VT | FF)+    -- one TextSpace
escape     ::= "\" ( "{" | "}" | "$" | "%" | "&" | "#" | "_" | "\\"-as-textbackslash* )
ctrl_space ::= "\ "                                     -- also "\" followed by a newline or tab
group      ::= "{" text_body "}"                        -- grouping only, no output
nested     ::= ("\text" | "\mbox" | "\textrm" | …) ws* "{" text_body "}"
math_cmd   ::= "\(" … "\)" | "$" … "$"                  -- inline math inside text
line_break ::= "\\"                                     -- see rules
comment    ::= "%" not-newline* newline?                -- discarded, like TeX
```

\* `\\` inside text is a line break in LaTeX, not a backslash. We have no
line layout in text, so the choice is to **reject** it with a typed error
(below). The way to write a backslash is `\textbackslash`.

### 2.2 Rules

R1. Text bodies are lexed by a dedicated text lexer, entered when the math
parser reads a text command's `{`. No string pass runs over them, before
or after.

R2. Whitespace runs collapse to one `TextSpace`. Leading and trailing spaces
of the body are kept (TeX does). `\ ` is a `ControlSpace` item, different from
`TextSpace`.

R3. Control words inside text swallow the spaces after them (TeX rule). The
only control words accepted inside text are: the nested text commands,
`\textbackslash`, `\ldots`/`\dots`, `\quad`/`\qquad`, and the known text
symbol table. Anything else gives `ParseError::Unknown(name)`. It is never
stored as a literal string.

R4. `\{ \} \$ \% \& \# \_` become `Char` items for that character, with
`escaped: true` dropped. They are characters. The printer re-escapes every
char in the special set (2.4), so this needs no extra AST bit.

R5. `\(…\)` and `$…$` inside text open a math sub-parse, stored as a child
`MathNode`. A stray `\)` or `\(` with no partner gives
`Malformed("unmatched \\) in text")`. **`\)` is never a literal `)`.**

R6. A nested text command gives a child `TextRun` with its own style. The
nesting depth counts toward `max_depth`.

R7. A bare `{…}` group in text has no meaning. Its items are spliced into the
parent and leave no node behind.

R8. Inside text, `^ _ & ~` and the `#` char are errors, unless escaped.
`~` is the exception: it is a `NoBreakSpace` item.

### 2.3 Errors (all typed, never panics)

| Case | Error |
|---|---|
| Missing `{` after the text command | `Malformed("expected '{'")` |
| EOF before the closing `}` | `Malformed("unmatched '{'")` |
| Stray `}` cannot happen (it closes the arg) | n/a |
| `\` at EOF | `TrailingBackslash` |
| `\\` in text | `Malformed("line break in text")` |
| Unknown control word | `Unknown(name)` |
| `^`, `_`, `&`, bare `#` | `Malformed("unexpected token X in text")` |
| Unmatched `\(`, `\)`, `$` | `Malformed("unmatched math shift in text")` |
| Depth over the limit | the existing depth error |

### 2.4 AST

```rust
MathNode::Text(TextRun, TextStyle)
struct TextRun(Vec<TextItem>);
enum TextItem {
    Str(String),          // maximal run of plain chars (no whitespace)
    Space,                // collapsed inter-word space
    ControlSpace,         // "\ "
    NoBreakSpace,         // "~"
    Symbol(&'static str), // \textbackslash, \ldots, …
    Nested(TextStyle, TextRun),
    Math(Box<MathNode>),
}
```

Invariants (checked by the constructor, and assumed by the printer): no two
adjacent `Str`. No `Str` is empty. `Space` is never adjacent to `Space`.
`Str` holds the exact chars, so `{` in `Str` came from `\{`. Escapes are not
stored as tokens, because there is exactly one spelling for each char.

### 2.5 Canonical print

```
print(Text(run, style)) = "\" style_cmd "{" print_run(run) "}"
print_run: Str s       -> each c: if c in  \ { } $ % & # _ ^ ~  then
                            the escape (\textbackslash{}, \textasciicircum{}, \textasciitilde{} for the 3 without a symbol)
                          else c
           Space        -> " "
           ControlSpace -> "\ "
           NoBreakSpace -> "~"
           Symbol n     -> "\" n "{}"            -- the empty group ends the word, so no R3 swallow
           Nested(st,r) -> "\" cmd(st) "{" print_run(r) "}"
           Math m       -> "\(" print(m) "\)"
```

Properties: `parse(print(t)) == t` for every `t` that `parse` returns.
`print(parse(print(t))) == print(t)` (idempotent). The printed text body
contains no bare `( ) [ ]` that math-level code can see, because math-level
code never looks inside a text arg (R1 and 3.2). It also contains no `{ )}`
style byte sequence that matters, because no string rewrites run (3.1).

## 3. Rebuild plan

3.1 **Delete** from `preprocess`: P2-P7 (move `{(}`-style delimiter
unwrapping into the math parser, where a `BeginGroup` holding one delimiter
char and optional spaces becomes the bare delimiter), P11 (`\mbox` becomes an
alias in the command table, drive.rs:1417 already accepts it), and P8, P9,
P10 (become parser rules: a `\rm`/`\bf`… font switch at the head of a group
styles the rest of the group; `\over` and `\choose` become infix operators
handled in the group parser, which already has the rows). Only `trim_math`
(P1) is left, and it could also move into the tokenizer. `replace_unescaped`
and `ends_with_odd_backslash` are then deleted. `preprocess` stays as a
public function, for API compatibility, as identity plus trim.

3.2 **Lexing.** The tokenizer becomes lazy, or gets a mode switch: when the
parser reads a text command, it gives a char cursor (the byte offset of the
`{`) to `text::lex_body`. That lexes to `TextItem`s by 2.1/2.2 and returns the
offset after the matching `}`. Math tokens are never built for text bodies.
As a stopgap that is compatible with the current pre-tokenised design, the
tokenizer keeps `Token::TextArg(String)` raw spans for text commands, so the
token stream contains nothing inside text.

3.3 **Lookahead.** `script_follows_parens` (parse.rs:396) skips over
balanced `BeginGroup…EndGroup`, and returns false on an unbalanced
`EndGroup`. With 3.2 in place, text is one opaque token anyway.

3.4 **Printer.** Replace `push_escaped` for text with `print_run` (2.5).
Remove the `\mathrm{\text{…}}` double wrap (ast.rs:1708-1716) by making
`TextStyle` carry its own command. Keep `push_escaped` for `\operatorname`
names (ast.rs:1957, 1991), but restrict it to the chars that are legal there.

3.5 **collapse_text** merges `TextRun`s by concatenating items and
re-normalizing (the Str/Space invariants).

3.6 Gold files whose output has text in it change (`\text {` spacing,
escapes). Regenerate them in one reviewed commit.

### Property tests

`proptest = "=1.5.0"` is already a dev-dependency (Cargo.toml:46), and it is
used in tests/roundtrip.rs and tests/prime_depth_props.rs. No new dependency
is needed.

* Generator, from the grammar: a recursive `prop_compose!` strategy for
  `text_body`, with leaves `char` (weighted to the specials `( ) [ ] { } ' ^`
  and ASCII letters), `space`, each `escape`, `ctrl_space`, `~`, and nodes
  `group`, `nested` (depth ≤ 3), and `math_cmd` (a small math expression with
  `( ) ' ^` atoms). Wrap each body in math context: `X`, `(X)`, `(X)'`,
  `X^2`, `{(}X{)}`, and `X\ ` at the end.
* Properties:
  (a) `parse` never panics.
  (b) For accepted `s`: `parse(print(parse(s))) == parse(s)`.
  (c) `print` is idempotent.
  (d) A generated `TextRun` value goes through `parse(print(…))` unchanged.
      This is the AST-first direction, which covers the printer escapes.
  (e) For the error rows of 2.3, a mutated generator checks that the error
      variant is the typed one.
* Use a fixed `ProptestConfig { cases: 4096, rng_algorithm: ChaCha, … }` with
  a fixed seed in CI for reproducibility, plus an env override for long runs.

Regressions: every `fuzz/corpus/parse/roundtrip-g1-*` … `roundtrip-g9-*`
seed (34 files: g1 ×8, g2 ×1, g3 ×2, g4 ×10, g5 ×3, g6 ×4, g7 ×3, g8 ×2,
g9 ×1), plus `control-space-comment` and `prime-run-34`, is loaded by a
table-driven test that runs property (b) on each file. The hand-written tests
in tests/fuzz_roundtrip.rs (G1-G9, including
`paren_lookahead_skips_text_bodies`) stay. Expected outputs change only where
2.3 now rejects input that used to be silently mangled (`\\` in text, unknown
control words in text). Those cases move to an `is_err` table.
