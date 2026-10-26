# sv-howto-skelter: reviser log

Checked against svelters `6c1ea458` and cadatomic `b350c24` (unchanged since stage 1). Nothing built.
Prose length of draft1: ~1,950 words (draft0 ~1,670); code blocks extra.

## Pass 1 — verified fixes (draft0 → draft1)

1. Fragment merged twice — fixed. Steps 3 and 4 now assign `cfg.spec_fragments = vec![fragment];` (source/mod.rs:113-117, 352; app_values.rs:80); package.json `specFragments` is presented as the alternative for a pre-written fragment, with "don't use both" and the exact error.
2. `"widget": "placeholder"` in fragment — fixed. Removed; the `unknown_element` item now says a tag missing from the fragment's `"tags"` means `register` did not register it (spec.rs:425).
3. `--sv-list-hover` undefined, box invisible — fixed. Confirmed: svelters-skelter adds no token sheet (its only `add_sheet` is the component style op, style.rs:56), computed background/border default to transparent (computed.rs:107-125). The example now uses literal `border`, `background-color` and `color`; one sentence explains why and gives the token-sheet line (`d.add_sheet(&Theme::default().css())`, the API cadatomic uses in theme/skelter.rs:151-182). Added a troubleshooting note for transparent boxes.
4. `draw_text` draws nothing without a shaper — fixed. Added `backend.with_doc(|d| d.set_text_shaper(Box::new(skelter_text::TextSystem::new(fonts.clone()))));` to `main`, marked as not in the stock runner (TextSystem implements TextShaper, system.rs:674-681; cadatomic sets one, cadatomic-app/src/skelter/mod.rs:81). The kind's `record` now actually draws its value and unit, adapted from skelter-core's `TextKind::record` (testkinds.rs:184-189), so the reader sees the fix work; added a "box draws but its text doesn't" troubleshooting item.
5. Member-id type part — fixed: prop type / signal arg type / method return type, `void` when none (kind.rs:234-245).
6. `Run` block didn't compile — fixed: added the `use` lines, `type Calls`, and the `calls`/`c` lines from tests/behaviors.rs:13-19, 41-43.
7. Placement of behaviour calls — fixed: register after `install_widget_services`, before `Embed::with_backend`; collect `behavior_modules()` there and write them after `cfg`, before `SourceProvider::new`.
8. No dependency block — fixed: full `[dependencies]` with paths, plus the AccessKit `[patch.crates-io]` and the rquickjs-sys profile, stated as copied from svelters' root manifest (sv/Cargo.toml:114-128). `skelter-widgets` named where needed (theme sheet, widgets interface).
9. File locations — fixed: `main` goes in the reader's binary crate; `ui/` holds `package.json` and `src/App.svelte`; run from the directory holding `ui/`.
10. Commands not runnable / no starter app — fixed: `cargo run -p … --bin …` forms, `--check`, the stale usage text note, "svelters has no skelter starter app", a minimal package.json (marked as an example). Correction to the verifier's suggestion: the app needs **no** `node_modules`; the loader falls back to the checkout's `packages/native-runtime` and resolves Svelte through its alias table, so the prerequisite is `pnpm install` in the svelters checkout (source/mod.rs:134-143; aliases.rs:23-46; cadatomic ui/package.json "nothing is installed here").
11. "kind … already defined" — fixed: the id is the derived kind name; both causes (double merge, built-in tag) and the separate `tag <x>` message are listed.
12. Collision messages — fixed: cross-kind and in-kind messages both named with the right meaning (registry.rs:68-73).
13. `register_builtin` claim / `shared` argument type — fixed: `Rc<RefCell<Document>>`; build the document on `kinds.registry()` or `register_builtin` plus your `register` (kinds.rs:148-161, 665-667).
14. "Steps 2 to 5 are the same" — fixed: the other steps are the same; step 2 uses `skelter_widgets::host::register::<K>(reg)?`, and the crate is needed.
15. `:state()` troubleshooting — fixed: symptom is "a `:state()` rule never matches"; silent in dev, the message only for a precompiled sheet.
16. "Only change is draw_box" — fixed (now accurate for the larger change: doc string shortened, `record` new, `QTY_UNIT` added and marked).
17. package.json not marked illustrative — fixed: the new block is labelled an example and is consistent with step 3 (no `specFragments`).
18. "No diagnostics" — fixed: "compiles without errors".
19. Tier 0/1 jargon — fixed.
20. cadatomic never introduced; different real kind — fixed: introduced once in the opening, with the stand-in note.
21. Broken backticks in headings — fixed: those two messages now use quotes; the `:state` one became a symptom heading.
22. "## Steps" — fixed: "## Add the `<quantity-field>` kind".

Rejected: none. One verifier suggestion amended (10, node_modules).

Length trims made only to stay within 2,000 prose words after the additions (shorter intro, merged sentences); no content removed beyond the duplicate definition of a kind in the prerequisites.

## Pass 1 — handprint (draft1 → draft2)

- Before: `draft1.md: tokens=2080 findings_total=13 high=4 med=9 doc-style high=0 med=0`
- After:  `draft2.md: tokens=2103 findings_total=7 high=1 med=5 doc-style high=0 med=0`
- Acted on: `punct.ellipsis_char_*` (high): my own `…` elisions in two quoted messages became `...`, as draft0 had them. `punct.en_dash_rate` (high): "0–31"/"32–63" became "0 to 31"/"32 to 63", same values. `biber.synthetic_negation_rate`, `biber.pron.indefinite`, `punct.contraction_rate`: rephrased about ten "don't/doesn't/isn't/nothing/something" sentences.
- Skipped: `biber.downtoner_rate` (high): its spans are the word "kind", the page's subject and a Rust identifier (the lexicon reads it as "kind of"). `md.inline_code_rate`: identifiers in inline code are this page's convention. `read.*`/`syn.ly_adverb_rate` "increase": not worth forcing.
- Problem found in review: one rewrite ("svelters ships skelter examples only as tests") was not a fact from the code, so it was reverted in pass 2. Its "only" also raised the downtoner count.

## Pass 2 — handprint (draft2 → draft3)

- Before: `draft2.md … high=1 med=5`; after: `draft3.md: tokens=2113 findings_total=7 high=1 med=6`
- Acted on: `biber.modal.possibility` / `lex.hyland.cat.hedges` (increase): three "you can" where the option really is optional (`register_with`, the theme sheet, checking the fragment). Removed the "only" I had added. Wrote the usage-text note without speculation ("omits `skelter`").
- Reverted: the starter-app sentence went back to draft1's wording, so synthetic negation went back to medium.
- Skipped: downtoner ("kind"), inline code, ly-adverbs, as before.

## Pass 3 — handprint (draft3 → draft4)

- Before: `draft3.md … high=1 med=6 (incl. synthetic_negation)`; after: `draft4.md: tokens=2116 findings_total=7 high=1 med=6 doc-style high=0 med=0`, with synthetic negation gone.
- Acted on: the starter-app fact, restated without negation and checked in `sv/examples/`: every example is egui, gtk, qt, appkit or winui ("svelters' example apps target its other backends").
- Meaning repairs after diffing (see below): three pass-1 rewrites drifted, so I fixed them so they say what draft1 said, without adding negation.
- Skipped: `punct.quote_single_straight_rate` (from possessives of names like svelters'/skelter-core's), `punct.comma_rate` (slightly over; it comes from identifier lists), downtoner, inline code, and hedges beyond the honest ones. More hedging would weaken instructions that are exact.
- Churn: passes 2 and 3 report the same families as pass 1 minus those fixed. Nothing new appeared, so I stopped at 3.

## Chosen version: final.md = draft4

draft4 has the fewest High/Medium findings in the families I can legitimately act on: none remain in punctuation, negation or indefinites. The remaining high finding (downtoner) is caused by the word "kind" and cannot be fixed without renaming the subject. draft2 had one fewer medium, but it contained the invented "examples only as tests" claim, so it is not a valid choice. Self-check for AI tells: the headings name subjects, nothing restates a list, there are no filler intensifiers or "It's <adj> to" openers, and the page ends on troubleshooting with no closing summary.

## Diff check: final.md vs draft1.md

- Code blocks: identical (fenced content compared programmatically).
- Facts, labels, values, identifiers: unchanged. "0–31"/"32–63" now read "0 to 31"/"32 to 63" (same values), and `…` in two quoted messages is now `...` (an elision in both versions; the messages themselves are unchanged).
- Sentences whose meaning drifted and were fixed back:
  - "until Svelte or the kind sets it" was narrower than "until something sets it" → now "until a prop write sets it".
  - "Your `call` leaves that method to the default" misdescribed the case (the kind does not override `call`) → now "Your kind keeps the default `call`, which rejects every method."
  - "A development build reports it silently, by dropping the rule" was self-contradictory and also wrong (the rule is kept and never matches) → now "A development build stays silent and the rule fails to match".
- The starter-app sentence changed wording but not its fact: svelters has no skelter app in `examples/`.
- Prose: about 1,980 words.
