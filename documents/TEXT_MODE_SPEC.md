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
nesting depth counts toward `max_depth`. Text that holds inline math that
holds text again is also capped at 32 text levels (`TEXT_LEVELS`), even when
`max_depth` is raised: `Malformed("text nests deeper than 32 levels")`. This
bounds the text → math → text recursion, whose sub-parses are not on the
heap-stack depth model.

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
           Math m       -> "\(" print(m) "\)"   -- "$" print(m) "$" when print(m) has a
                                               top-level "\)", "\({" print(m) "}\)" when
                                               it also has a "$"
```

Properties: `parse(print(t)) == t` for every `t` that `parse` returns.
`print(parse(print(t))) == print(t)` (idempotent). The printed text body
contains no bare `( ) [ ]` that math-level code can see, because math-level
code never looks inside a text arg (R1 and 3.2). It also contains no `{ )}`
style byte sequence that matters, because no string rewrites run (3.1).

## 3. Rebuild plan

Section 4 lists where the build differs from this plan. Those choices are
the spec.

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
A `MathNode::Text` prints by its style:

- `TextStyle::Text`: `\text{` print_run `}`.
- a math font (`Rm`, `Bf`, `It`, `Sf`, `Tt`, `Bb`, `Cal`, `Frak`, `Scr`,
  `Boldsymbol`, `Pmb`) whose run is only ASCII letters: `\mathX{` run `}`.
  (An earlier build printed `\mathX c` for one letter; spec 5, Q2 braces
  every command argument, and a braced argument now costs no extra depth.)
- a math font whose run is anything else: the double wrap
  `\mathX{\text{` print_run `}}`. A math font argument is math, so spaces,
  digits and punctuation in the run would be re-read as math atoms; the
  inner `\text` keeps the run opaque. The double wrap stays (the original
  plan to remove it by giving `TextStyle` its own text command is dropped).

Keep `push_escaped` for `\operatorname` names, but restrict it to the chars
that are legal there.

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

## 4. As built: decisions that are now the spec

These were open or different in sections 2 and 3. The code and tests follow
them.

4.1 **Top-level `\textrm`, `\textbf`, `\textit`, `\textsf`, `\texttt` stay
math fonts.** In math, they parse as before, to `MathNode::Text(run, Rm|Bf|…)`
by the math font table. Only inside a text body are they nested text runs
(`TextItem::Nested`). This keeps every existing gold and caller of the math
font styles unchanged.

4.2 **The `\mathX{\text{…}}` double wrap is kept** for a styled run that is
not only ASCII letters (3.4). Removing it would need a text command per
math font, and `\mathbb` / `\mathcal` / `\mathfrak` have none.

4.3 **A 32-level text cap** (`TEXT_LEVELS`) bounds text → inline math → text
recursion on top of `max_depth` (R6).

4.4 **`ends_with_odd_backslash` stays.** `preprocess` keeps `trim_math`, and
`trim_math` needs `ends_with_odd_backslash` to keep the space of a trailing
control space `\ `. 3.1 said both would go; only the string rewrites did.

4.5 **`\its` is one control word.** Font scopes are a token rule
(`group_rules::font_scopes`), so `{\its …}` is the unknown command `\its`,
not `{\it s…}`. The old string rewrite matched the prefix. Two G4 fuzz inputs
that relied on it moved to the rejection table
(`Err(Unknown("\\its"))`, `Err(Unknown("\\itsM"))`).

4.6 **Packaging.** This file is in the `include` list of `Cargo.toml`, so the
README link resolves in the published crate.

4.7 **Text math print.** A `\)` at the top level of an inline-math body would
end `\(…\)` early, so such a body prints as `$…$` (or `\({…}\)` if it also has
a `$`), per 2.5. Seed `roundtrip-canon-text-math-close`.

## 5. Printer grouping spec

Status: approved by Paul on Oct 14 with Q1 and Q2 (5.6), and built (see
5.7). It replaces
the case-by-case brace rules in `src/parser/ast.rs` (`bare_arg`,
`closes_open`, `paren_need`, `option_needs_group`, the fence wrap, the
text-math `$` choice) with one decision. The acceptance gate is the
canonical-print property (`tests/canonical_print_props.rs`) and the depth
property (`tests/depth_economy.rs`), with every seed in
`documents/FUZZ_LEDGER.md` kept as a regression.

### 5.1 Why the patches keep coming

Of the 64 ledger rows, 30 are "printer canonical form". At least 10 of
those came after the brace economy (133dd49), and every one is the same
mistake: the printer decides "bare is safe" by looking at **the child
node**, but the parser decides what a token means from **the token stream
around it**. There are three places where that context leaks out of a
node, and the printer models none of them in general:

1. **The paren lookahead is non-local.** `script_follows_parens`
   (parse.rs:408-446) starts at any `(` that begins an atom. It scans raw
   tokens forward, skips only `{…}` groups, counts `(`/`)` across command
   arguments, `[…]` options and sibling nodes, and asks whether the
   matching `)` is followed by `^`, `_`, `'` or a superscript character.
   So a bare `(` or `)` anywhere later at brace level 0 changes how an
   earlier `(` parses. This one rule explains primed-close, primed-open,
   option-paren, `\left((\right)` and the `(` base cases.
2. **Some groups are not groups.** `unwrap_delimiters`
   (group_rules.rs:18-36) turns `{[}`, `{]}`, `{ (}` and `{ )}` into bare
   tokens. `font_scopes` and `convert_infix` (group_rules.rs:143) rewrite
   `{\rm …}` and `{… \over …}`. A `{…}` the printer adds is only a group if
   none of these rewrites applies to it.
3. **Whole-input and text-math rewrites.** `strip_fences`
   (parse.rs:681) removes a `\(…\)` or `\[…\]` around the whole input and
   rejects an unclosed one. Text math ends at the first `$` at depth 0
   (text.rs:205) or at `\)` (text.rs:228).

Hypothesis: the printer keeps producing non-canonical output because
grouping is decided per node kind, but the hazards are properties of the
**token sequence**. The fix is to decide grouping on tokens, using the
parser's own rules, in one place.

### 5.2 The rule

Print each child bare first, as a token list `T` (the tokenizer's tokens,
not a string). Group it as `{T}` when any of these holds; otherwise keep it
bare.

- **G1 (reader).** In the child's context `C` (5.3), the parser's reader
  for `C` would not consume exactly `T` and return the child. This is
  decided by the table in 5.3, which is a static summary of the reader.
- **G2 (paren balance).** `T` has an unbalanced `(` or `)` at brace level
  0, and the child is not a row item handled by 5.4. A child whose parens
  are balanced at level 0 cannot pair with any paren outside it. (A
  balanced `(…)` inside `T` is already the child's own business. Whether it
  becomes a paren script is decided when that child was printed.)
- **G3 (fake group).** If the group would be `{[}`, `{]}`, `{ (}` or
  `{ )}`, use two groups (`{{[}}`) or print `(`/`)` without the space
  (`{(}` is a real group: group_rules.rs:21-23 only match the spaced
  form).

A group adds one level of depth (drive.rs:321 `push` with `counts`; the
group's list costs 1). G1-G3 group only when the bare form would reparse
differently, so the print is never deeper than a source that parses to
the same tree. That is the depth invariant.

### 5.3 Contexts (G1)

Each row says what the reader does and so when a bare `T` is wrong.

| context | reader (file:line) | bare `T` reparses differently when |
|---|---|---|
| **command argument** (`\frac`/`\binom` args, accents, `\not`, `\sqrt` body, `\color`-family bodies, `\phantom`, `\cancelto`, over/under args) | `step_arg` drive.rs:462-486. `{` gives a group, `(` gives a **paren group** (`Kind::paren`), anything else gives one nucleus (drive.rs:540-590) | `T` is not exactly one nucleus: more than one token-level item, starts with `(` (it would read `(…)`), starts with `'`, `^`, `_` or a superscript char ("script has no base", drive.rs:550, 593), or is a nucleus followed by its own scripts |
| **argument right after a `[…]`-taking command** (`\sqrt`, `\xrightarrow`, `\xleftarrow`) | the command peeks for `[` (drive.rs:1340, 1597) | as above, plus `T` starts with `[` |
| **optional argument `[…]`** | `Kind::list(Stop::index())`, stops at `]` at group level (parse.rs:399, drive.rs:1050-1060) | the parser, asked directly (`option_ends_early`, which runs `drive::parse_option_body` over `T` plus the closing `]`), does not consume `T` exactly up to that `]`. A static scan is wrong here (`{]}` unwraps to a bare `]`, and a nested option command's own `]` closes its own option). An empty row prints as nothing, because `[]` is an empty row and `[{}]` costs one more level |
| **script (sub or sup)** | the script reads one argument via the same `step_arg` path (drive.rs:755-800) | same as a command argument |
| **script base** | the base is the previous nucleus (drive.rs:494-540). A `(` base starts a lookahead (drive.rs:494-497) | `T` is not one nucleus. `T` ends in `)` that matches a `(` inside `T` (that would become a paren-script base instead of the tree's base). `T` itself ends in a prime superscript (`x'` then `^`: the marks join, drive.rs:765-768, push_prime drive.rs:854) |
| **prime marks** | `'` is script signal 3 (drive.rs:767). A run stays one flat row (drive.rs:854) | a ′ superscript prints as `'` only when the base is not already primed in a way the reader would merge, and only when not `prime_symbol` (`'` reads back as `\prime`). Otherwise `^{′…}` |
| **row item** | the list reader takes nuclei in turn. Adjacent ASCII letters join into an operator word with `bare_operators` (drive.rs:603) | the item is a nested Row (it would flatten). It is a bare `'`, `^`, `_` or superscript char (no base). Two letters touch under `bare_operators` (use a space, not a group). Paren rule 5.4 |
| **`\over` / `\choose`** | `convert_infix` group_rules.rs:143 rewrites `{a \over b}` before parsing | the printer never emits `\over`. Fractions print as `\frac`. A Row containing a bare `\over` token cannot arise from a tree, so no rule |
| **whole print** | `strip_fences` parse.rs:681 | the print starts with `\(` or `\[` and is longer than that token: group the whole print. Applies **only** to the outermost print, never to a nested body (the 3f5f2d6 bug) |
| **text math** `\(…\)` / `$…$` inside `\text` | text.rs:95, 205 (first `$` at depth 0 ends `$…$`), text.rs:228 (`\)` closes) | body has `\)` at brace level 0: use `$…$`. It also has `$` at level 0: group the body. Decided on the body exactly as written (unwrapped) |
| **dimensions** (`\rule`, `\hspace`) | `collect_group_text` parse.rs:587, `parse_tex_dim` parse.rs:1232 | not a grouping question: always `{…}`. The printer must emit the grammar `[+-]?digits[.digits]` + unit, which d5f097d now enforces on input. A NaN `Dim` must be unrepresentable from a parse |
| **`\left` / `\right` / `\big` delimiters** | `parse_delimiter`. A `(` after them is a delimiter, not a lookahead start | delimiters are never grouped. Their `(`/`)` do not count for G2 or 5.4 (the scan in parse.rs:408 does count them. See open question 5.6.1) |

### 5.4 Row items and the paren lookahead

For a row, the printer runs the same scan as `script_follows_parens` over
the **emitted token stream**, not over nodes:

1. Emit items bare into a token buffer, tracking brace level.
2. For every `(` at brace level 0 that begins an atom, find its match by
   the parse.rs:408 rules (skip `{…}`, count every other `(`/`)`), and
   check whether the matching `)` is followed by a script signal.
3. Compare with the tree. A paren-script exists exactly where the tree has
   `Superscript/Subscript/SubSup(Delimited-paren-group …)` from
   `Kind::paren`. Where the scan would see a paren-script that the tree
   lacks, group the **smallest item that contains the matching `)`**.
   Where the tree has one that the scan misses, the item was printed wrong
   and that is a bug.
4. Repeat until stable. Grouping only hides tokens, so this terminates.

This replaces `open_parens`, `closes_open`, `paren_need`, `scripted_paren`,
`peel_primes`, `ends_with_paren` and the paren half of
`option_needs_group`. The scan uses the parser's own rules, so a later
change to the lookahead changes the printer with it.

### 5.5 Implementation sketch

- Print to tokens: `Vec<Token>` through the existing tokenizer types, and
  render to a string at the end.
- One function, `needs_group(ctx: Ctx, t: &[Token]) -> Group` with
  `Group::{Bare, One, Two}`, implementing G1-G3 from the 5.3 table. Plus
  one row pass for 5.4.
- Keep `to_latex_with(&ParseOptions)`, because `prime_symbol` and
  `bare_operators` change the reader.
- Tests: every seed in `fuzz/corpus/parse/roundtrip-*` and every
  `FINDINGS` row stay. Add a unit test per 5.3 row, citing the line. The
  canonical-print and depth properties at 50,000 are the gate.

### 5.6 Decisions (were open questions)

1. **Q1, approved: the lookahead skips delimiter tokens.** `paren_script_match`
   (parse.rs, used by `script_follows_parens`) skips the token after
   `\left`, `\right`, `\middle` and the `\big` family. This is a parser
   change, and it has a CHANGELOG entry.
2. **Q1, approved: the lookahead skips `[…]` options** of `\sqrt`,
   `\xrightarrow`, `\xleftarrow`, `\color`, `\textcolor`, `\colorbox` and
   `\fcolorbox`. So G2 no longer applies inside options; only a top-level
   `]` groups an option. Refinement (fuzz r10 and r15, general rule): a
   lookahead belongs to the list it starts in and stops at any token that
   ends that list: `&`, `\\`, `\end`, `\right` or an option's `]`
   (`Parser::stop`, the `Stop` of the list `step_list` is reading, passed to
   `paren_script_match`). r10 was the `]` case (`\xleftarrow[(]))^\x08`); r15
   was `&` and `\end` (`\begin{vmatrix}{(}&{)^x}\end{vmatrix}`, where a `(`
   in one cell paired with a `)` in the next). Without it a braced body let
   an inner `(` pair with a `)` beyond the end of its list.
3. **Q2, approved: bare command arguments still parse, but print braced.**
   Correction to the earlier text of this item: TeX and KaTeX do accept
   single-token arguments such as `\frac12`. What TeX does not read as
   intended is an argument that is itself a command with arguments
   (`\frac 1\frac 12`): it is not one token. The parser keeps reading all of
   these (MathJax does too). The printer now always braces a command
   argument (`\frac {1}{2}`). Script arguments stay bare-first (G1:
   `x^a`, `x^\alpha`, `x^\frac {1}{2}`).

### 5.7 As built

- **Depth model (parser, Q2).** A `{…}` group that is itself a command or
  script argument costs no extra level (drive.rs `GroupPhase::ArgOpen`).
  `\frac{a}{b}` and `\frac ab` cost the same, so always bracing never
  nests deeper than the source. The depth property holds at 50,000 cases.
- **G1** is `bare_arg` (the static summary of the nucleus reader). It is
  used for script arguments and script bases. Command arguments skip G1,
  because they are always braced (`write_arg`).
- **G2** is satisfied structurally. With Q1 and Q2, a bare level-0 paren
  can only come from a bare `(`/`)` atom or a script base, and the row
  pass handles both. `bare_arg` keeps `(`, `)` and `[` out of script
  arguments.
- **G3** is applied where a fake group would change the tree: a lone `[`
  right after an option-taking command (`\sqrt {{[}}`), and a lone `]` in
  an option (`[{{]}}]`). Elsewhere `{[}` reading as a bare `[` gives the
  same tree, so one group is kept, which keeps depth equal to the source.
- **Row paren pass** (`paren_groups`, ast.rs). It tokenizes the row as
  printed (tokenize + group_rules, as the parser does) and runs the
  parser's own `paren_script_match` from each bare `(` that starts an item.
  It groups the item that owns the match, then repeats. Paren-script
  groups always print braced (`{(a)}^2`), so any match is a hazard.
- **Deleted helpers:** `open_parens`, `closes_open`, `peel_primes`,
  `ends_with_paren`, `scripted_paren`, `paren_need`,
  `after_delimiter_command`, and the paren half of `option_needs_group`
  (the static `]` scan is gone, replaced by `option_ends_early`, which asks the parser).
- **Cost invariant (5.4):** printing is linear. A subtree is printed once per
  level, never twice (row paren pass, options, text math each had a 2^k
  variant, fuzz r11 and r12).
- **Unchanged:** a command name is still followed by one space
  (`\frac {1}{2}`); the primed-base and prime-mark rules; the whole-print
  fence group; the text-math `$` rule.
