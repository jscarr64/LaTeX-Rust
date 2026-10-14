# Fuzz and property ledger

One row per crash or property failure found so far. "Minimal input" is the
regression seed in `fuzz/corpus/parse/`, with control characters written as
`\u{…}`. The fixing commit is the commit that made the seed pass, taken from
`git log` of the seed and its test (the red commit adds the seed, the next
commit for that id fixes it).

Notes:

- G3, G7 and G8 were first fixed in the old `preprocess` string rewrites (the
  SHAs below). The text-mode rebuild (3ed090f, 2ccb7f2) later removed those
  rewrites; the seeds still pass.
- G9: at 3ed090f a `\)` inside text became a typed error, so the seed is
  rejected instead of crashing. a3c0b85 (the `(` lookahead skips brace
  groups) is the second half of that fix.
- prime-run-34: 758452c made a prime run one flat row; a5a3623 made it print
  flat, which is what fixed the reprint depth.
- The 15 depth rows are fuzz crashes that also failed at c846b58 (before the
  text rebuild). Fixed by the printer brace economy, 133dd49.
- canon-*: found by the whole-grammar canonical-print property or by fuzzing
  after 133dd49. `canon-sqrt-bracket` and `canon-option-bracket` also fail at
  c846b58. `canon-fence-open` and `canon-text-math-close` were found by
  fuzzing 133dd49/f97bd29 (the fence case is a regression of the brace
  economy, the text-math case dates from the text rebuild).
  `canon-primed-subsup` and `canon-prime-symbol` were regressions in the
  uncommitted brace-economy work, caught by the property before they were
  committed; `canon-prime-symbol` fails only with `prime_symbol` on.
- canon-primed-close*: two fuzz crashes in the final 1800 s run on 2267377
  (a primed `)` after an unclosed `(`); both pass at c846b58, so they are
  regressions of the brace economy, fixed in 68ecd89.
- canon-primed-open: fuzz crash in the 1800 s run on b6919b9, the mirror case
  (a primed `(` base was not counted as open); fixed in 77a271b.
- canon-option-paren*: fuzz crashes in the 1800 s run on e5d6f1e (a `(` or
  `)` in an optional argument paired with an outer `(`); fixed in ee73242.
- canon-primed-open and canon-option-paren* pass at c846b58: regressions of
  the brace economy.
- canon-text-math-fence: fuzz crash in the 1800 s run on 155c728. The
  `$…$` choice for text math looked at the fence-wrapped print (2267377)
  while the body was written unwrapped; fixed in 3f5f2d6.
- dim-dot: fuzz crash in the 1800 s run on 8742778. A dimension with no digit
  parsed to NaN (present since the parser was written); fixed in d5f097d by
  rejecting it in `parse_tex_dim`.

| id | minimal input | root-cause class | fixing commit | regression seed |
|---|---|---|---|---|
| G1 41ad61cb | `\mbox{\$}` | printer canonical form | 3420c9d | `roundtrip-g1-41ad61cb` |
| G1 a83b1d95 | `\text{\&}` | printer canonical form | 3420c9d | `roundtrip-g1-a83b1d95` |
| G1 b71856d2 | `\text{\u{0}\&\u{0}}` | printer canonical form | 3420c9d | `roundtrip-g1-b71856d2` |
| G1 ba56776f | `\mbox{\$\u{0}}` | printer canonical form | 3420c9d | `roundtrip-g1-ba56776f` |
| G1 c23638fb | `\u{0}\mbox{t\${}?}l(` | printer canonical form | 3420c9d | `roundtrip-g1-c23638fb` |
| G1 db72c70e | `\u{0}\mbox{\$\u{0}}` | printer canonical form | 3420c9d | `roundtrip-g1-db72c70e` |
| G1 e13c26ab | `\u{0}k'6\mbox{\#Q\u{0}\$\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}}oxs\u{10}` | printer canonical form | 3420c9d | `roundtrip-g1-e13c26ab` |
| G1 f7ab6a10 | `\u{0}\mbox{\u{0}\u{0}\u{0}\u{0}#Q\u{0}\$\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}}oxs\u{10}` | printer canonical form | 3420c9d | `roundtrip-g1-f7ab6a10` |
| G2 6da362ae | `\text{\\u{a}}` | text decode | 63ffcf8 | `roundtrip-g2-6da362ae` |
| G3 203b89fe | `(\text {\ ) }\u{0}(BB(BB` | preprocess rewrite | aeaf4d0 | `roundtrip-g3-203b89fe` |
| G3 51652d7b | `(\text {\ ) }\u{0}(BB` | preprocess rewrite | aeaf4d0 | `roundtrip-g3-51652d7b` |
| G4 3d55a54a | `h{\it{i'\text{  }<}<<}` | printer canonical form | 56a9404 | `roundtrip-g4-3d55a54a` |
| G4 3d80a1bd | `{\it\mbox{\u{2}}}` | printer canonical form | 56a9404 | `roundtrip-g4-3d80a1bd` |
| G4 63b40237 | `h{\it{rqi'\text{     }\ \<<<<} \<<<<}` | printer canonical form | 56a9404 | `roundtrip-g4-63b40237` |
| G4 84931cd8 | `{\it{\text{}}}` | printer canonical form | 56a9404 | `roundtrip-g4-84931cd8` |
| G4 9d223dd5 | `{\itsM\u{0}\mbox{{\u{2}{t\u{1f}h}}]}}Is` | printer canonical form | 56a9404 | `roundtrip-g4-9d223dd5` |
| G4 b6f7760a | `T{\it\text{[\u{e}\u{e}}}` | printer canonical form | 56a9404 | `roundtrip-g4-b6f7760a` |
| G4 b9ede5d3 | `\u{0}{\it\mbox{}}` | printer canonical form | 56a9404 | `roundtrip-g4-b9ede5d3` |
| G4 d3dde77a | `{\its\mbox{\u{2}P]}}` | printer canonical form | 56a9404 | `roundtrip-g4-d3dde77a` |
| G4 d807c16e | `{\it\text{}}` | printer canonical form | 56a9404 | `roundtrip-g4-d807c16e` |
| G4 fdf2f41c | `{\it{i'\text{}}}` | printer canonical form | 56a9404 | `roundtrip-g4-fdf2f41c` |
| G5 64e3983c | `-{\choose }` | other | aea104c | `roundtrip-g5-64e3983c` |
| G5 d3f2c752 | `{\choose}` | other | aea104c | `roundtrip-g5-d3f2c752` |
| G5 f091efdf | `` \u{0}\u{0}l-osem{\it\u{9}se\u{9}lAh\{\u{0}l-o`n\ U\u{1}e}{\it\u{9}\choose \ } `` | other | aea104c | `roundtrip-g5-f091efdf` |
| G6 5c1df81b | `$\u{b}$` | other | 5e6ebd1 | `roundtrip-g6-5c1df81b` |
| G6 75154f11 | `$$\u{a}\u{a}\u{a}\u{a}\u{a}\u{a}\u{a}\u{a}\u{a}\u{a}\u{c}\u{a}\u{a}\u{a}\u{a}\u{a}$$` | other | 5e6ebd1 | `roundtrip-g6-75154f11` |
| G6 85a23c0a | `\u{a}$\u{b}$\u{a}\u{a}\u{a} ` | other | 5e6ebd1 | `roundtrip-g6-85a23c0a` |
| G6 bb32f5b3 | ` $$ \u{c} $$  ` | other | 5e6ebd1 | `roundtrip-g6-bb32f5b3` |
| G7 2c53e50e | `\u{0}mb\mbox{I\choose'III\{itc}h` | preprocess rewrite | e03d192 | `roundtrip-g7-2c53e50e` |
| G7 62a0555f | `\u{1}\text{A\over{{\u{0}{\{\)}}}}` | preprocess rewrite | e03d192 | `roundtrip-g7-62a0555f` |
| G7 8569dea9 | `)\text{\over\u{0}{\{ }}` | preprocess rewrite | e03d192 | `roundtrip-g7-8569dea9` |
| G8 7b95ab68 | `\text{\{\ )}` | preprocess rewrite | 911515a | `roundtrip-g8-7b95ab68` |
| G8 f2ab310f | `(???????\text {\{\ ) }\ ) t{}Bb` | preprocess rewrite | 911515a | `roundtrip-g8-f2ab310f` |
| G9 1410dba0 | `(???????\text {\text {\ }\)') t{Lambda}Bb) }\ ` | text decode | 3ed090f | `roundtrip-g9-1410dba0` |
| control-space-comment | `\ %@QJXi` | preprocess rewrite | 2614042 | `control-space-comment` |
| prime-run-34 | `{[''''''''''''''''''''''''''''''''''}` | depth | a5a3623 | `prime-run-34` |
| depth 03a818ac | `lo_{==_{{lo_{=_{==_{=_{={lo_{=_{=_{{lo_\u{0}}<}a}}}}<}}a}}<}}` | depth | 133dd49 | `roundtrip-depth-03a818ac` |
| depth 1d3eba56 | `h{\choose\choose\choose\choose\choose\choose\choose\choose\choose a\choose}` | depth | 133dd49 | `roundtrip-depth-1d3eba56` |
| depth 2831ee77 | `\u{1e}{\choose\choose\choose\choose\choose\choose\choose\choose\choose"\choose}` | depth | 133dd49 | `roundtrip-depth-2831ee77` |
| depth 2ab0f7cc | `{\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over4}` | depth | 133dd49 | `roundtrip-depth-2ab0f7cc` |
| depth 32fcc59b | `\u{1e}{\choose\choose\choose\choose\choose\choose\choose\choose\choose\choose"}` | depth | 133dd49 | `roundtrip-depth-32fcc59b` |
| depth 3896821d | `{\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over-\over}` | depth | 133dd49 | `roundtrip-depth-3896821d` |
| depth 426209ab | `{\it\choose\choose\choose\choose\choose\choose\choose\choose\choose-o\choose}` | depth | 133dd49 | `roundtrip-depth-426209ab` |
| depth 46a6374d | `{\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over[\over}` | depth | 133dd49 | `roundtrip-depth-46a6374d` |
| depth 46dc6a92 | `{\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over4}` | depth | 133dd49 | `roundtrip-depth-46dc6a92` |
| depth 7225d264 | `\u{1e}{\choose\choose\choose\choose\choose\choose\choose\choose\choose\choose d}` | depth | 133dd49 | `roundtrip-depth-7225d264` |
| depth 8a7759eb | `{\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over-}` | depth | 133dd49 | `roundtrip-depth-8a7759eb` |
| depth a66a4405 | `lo_{==_{{lo_{==_{==_{=_{==_{{lo_{==_{={{}<}}}}<}}a}}}}<}}` | depth | 133dd49 | `roundtrip-depth-a66a4405` |
| depth aa588734 | `{\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over)\over}` | depth | 133dd49 | `roundtrip-depth-aa588734` |
| depth cbe83c33 | `{\it\choose\choose\choose\choose\choose\choose\choose\choose\choose g\choose}` | depth | 133dd49 | `roundtrip-depth-cbe83c33` |
| depth fa46a27f | `{\over\over\over\over\over\over\over\over\over\over\over\over\over\over\over4\over}` | depth | 133dd49 | `roundtrip-depth-fa46a27f` |
| canon-sqrt-bracket | `\sqrt{{[}}` | printer canonical form | 133dd49 | `roundtrip-canon-sqrt-bracket` |
| canon-option-bracket | `\xrightarrow[\mathrm{]}]{x}` | printer canonical form | f97bd29 | `roundtrip-canon-option-bracket` |
| canon-primed-subsup | `{x_{x}^ ′}^ ′` | printer canonical form | f97bd29 | `roundtrip-canon-primed-subsup` |
| canon-prime-symbol | `\textcolor{blue}{{x}^ ′}` | printer canonical form | f97bd29 | `roundtrip-canon-prime-symbol` |
| canon-fence-open | `{\( }x` | printer canonical form | 2267377 | `roundtrip-canon-fence-open` |
| canon-text-math-close | `\text{t $ \) $ u}` | printer canonical form | 2267377 | `roundtrip-canon-text-math-close` |
| canon-primed-close | `(^))'^)` | printer canonical form | 68ecd89 | `roundtrip-canon-primed-close` |
| canon-primed-close-accent | `(\not()'^x` | printer canonical form | 68ecd89 | `roundtrip-canon-primed-close-accent` |
| canon-primed-open | `('^))^\ ` | printer canonical form | 77a271b | `roundtrip-canon-primed-open` |
| canon-option-paren | `(^)\xleftarrow[)^x]]` | printer canonical form | ee73242 | `roundtrip-canon-option-paren` |
| canon-option-paren-open | `(^)\xleftarrow[(]x))^\u{8}` | printer canonical form | ee73242 | `roundtrip-canon-option-paren-open` |
| canon-text-math-fence | `\text{$\(\)$}` | printer canonical form | 3f5f2d6 | `roundtrip-canon-text-math-fence` |
| dim-dot | `\rule{2}{.}` | other | d5f097d | `roundtrip-dim-dot` |

## Count per class

| class | count |
|---|---|
| preprocess rewrite | 8 |
| text decode | 2 |
| printer canonical form | 30 |
| depth | 16 |
| other | 8 |
| **total** | **64** |
