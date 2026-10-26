# cad-howto-erc: reviser log

## Pass 1 — verified fixes

draft0 → draft1. Every verifier issue checked against code before applying.

1. Fixed. "Drawing disagrees with the circuit" has no place (checks.rs:560-574 has no `.at()`; reveal needs a spot, board/erc.rs:248). Table fix now says there is no place to jump to and what to look for; also added to the "no marker, clicking doesn't move the view" list in "Read a violation" (F70).
2. Fixed, with a better step than suggested: the user-facing route is the **Components library** pane's **Update** button on the "Out of date" part (en.ftl:1404-1434, lib_pane.rs:14), not a bare "Update from library" command (F71).
3. Fixed. "Before you start" says the tool keys work only with the pointer over the sheet; the single-pin steps click the sheet before Page Down, and each P/T key step starts "With the pointer on the sheet" (inventory Viewport layer, F72).
4. Fixed (kept, shortened). Says Copperfield is the ECAD engine under cadatomic and its tool isn't installed with cadatomic; adds that it prints full messages (unless `--summary`). Didn't add "build it from source": no user-facing build route for it is documented (F78).
5. Fixed. "What happens" now says the reason is saved but not shown in the list; dropped "so the next person to read the design sees it".
6. Fixed. Gap bullet rewritten without "kernel", "row's data", "draw"; dropped the "select the row to see the place" tail (it repeated "Read a violation").
7. Fixed. "Warning (from the connection matrix)" (F75).
8. Fixed. "If it appeared after an edit, undo that edit; the check reruns by itself."
9. Fixed. Ctrl (Cmd on macOS); "and, if the design has a board open, on the board" (F54).
10. Fixed. "(use a supply name other parts are already on, or the input becomes a net with only one pin)" (F76).
11. Fixed. Table lists port and sheet-entry drivers and notes all-passive nets aren't reported; "only" dropped from the power-port paragraph (F30, F77).
12. Fixed. Dropped "This is the error you will see most."
13. Fixed. Select the text first (Ctrl+A, Cmd+A on macOS); the pane opens as a tab over the sheet; click the sheet's tab to go back (F53 corrected).
14. Fixed (softened). "Most of its checks and their default levels come from Altium Designer's." No "inferred".
15. Fixed, and checked that the app's run reads the stored levels (erc.rs:179-185, F74): "A design imported from an Altium project keeps that project's levels, and those win."
16. Fixed. "(only the schematic, for findings with no place)".
17. Fixed, refined: the flag is dropped when the schematic's pane closes (hub.rs:655), so "until you close it" (the schematic) rather than "the design" (F73).
18. Fixed. Prose (outside the code block, table included) cut from ~2,300 to ~2,000 words, by tightening the table's fix column, removing the duplicated "Floating net labels" fix paragraph, the duplicated passive-pin reminder and the settings-page gap (already said above the table), and shortening the intro, junction note and power-port paragraph.

Fact sheet: F32, F65 and F68 citations corrected; F1 notes the hint is never shown; F53 and F54 corrected; F70–F80 added for the facts the fixes rely on. Style was not touched beyond these fixes; some sentences are left for the style passes.

## Pass 2 — handprint (draft1 → draft2)

- Before: `findings_total=31 high=5 med=13 doc-style high=0 med=0`
- After: `findings_total=28 high=2 med=11 doc-style high=0 med=0`
- Acted on:
  - `biber.synthetic_negation_rate` (high, 17.4 → 8.3): rewrote the negations in my own prose ("an input that nothing drives" → "left undriven", "If nothing is wrong" → "A clean schematic", "have no marker" → "listed without a marker", "Nothing drives the net: no …" → "The net lacks a driver: …", "touches no wire" → "stands apart from every wire", "did nothing" → "closed without waiving", heading "Inputs with nothing driving them" → "Undriven inputs and missing power"). The ones left in labels ("No ERC", "no-connect", "Nets with no driving source", "no violations") stay.
  - `md.bold_rate` / `punct.asterisk_rate` (high): took the bold off key letters (P then W …) and off the Error/Warning level words in prose, and turned the troubleshooting bullets, which had bold lead-ins, into plain "If …" paragraphs. Bold stays on exact UI labels (**Check (ERC)**, **Waive…**, **Remove waiver**, **Waived**, …).
  - `biber.downtoner_rate` (medium): cut the "only"s in my own prose ("work only with" → "need", "only the schematic" → "the schematic alone", "a net made only of" → "entirely"). The "only" in the label "Nets with only one pin" stays.
  - `biber.pron.indefinite` (medium): fixed by the same "nothing" rewrites.
  - Asides and parentheses: turned the palette parenthesis in step 1, the Menu-key parenthesis and "(next section)" into plain sentences.
  - `md.numbered_rate` (medium), in part: "Remove a waiver" is now one sentence instead of a two-step list. The "What happens" list became a paragraph.
  - Heading "Two outputs on one net" → "Conflicting outputs on one net", which clears `dev.headline.number_initial`.
- Skipped:
  - ellipsis findings: they come from the exact labels "Waive…" and "Tours…".
  - `punct.pipe_rate`, `md.table_row_rate`: the table.
  - `punct.slash_rate`, `md.inline_code_rate` (increase): a GUI page; adding slashes or code would be artificial.
  - `read.*` (increase): don't make the prose harder to read.
  - word-length bands and `biber.nominalization_rate`: these fall out of short, plain words.
  - `syn.ly_adverb_rate` (increase): I won't pad the page with adverbs.

## Pass 3 — handprint (draft2 → draft3)

- Before: `findings_total=28 high=2 med=11`
- After: `findings_total=27 high=2 med=10 doc-style high=0 med=0`
- Acted on:
  - `biber.synthetic_negation_rate` (medium, 8.3): the six remaining in my own prose ("has no settings page" → "lacks", "and no other kind" → "ignores every other kind", dropped "so it names nothing", "has no place to jump to" → "Clicking the row won't move the view", "has no wire" → "is left unwired", "produced no results" → "can't list any findings"). It left the reported list.
  - Length (brief): dropped "You don't need to press the button again." (the next sentence says it), the repeated "since the check runs on the active schematic", and a few words elsewhere. That gives ~1,550 words of prose plus ~460 in the table.
- Skipped:
  - `md.numbered_rate` (medium): the remaining lists are the how-to's required one-action steps.
  - `syn.ly_adverb_rate`, `read.*`, word length, ellipsis, pipes, slash, inline code: same reasons as pass 2.
- Churn guard: what remains is labels, the table, the required steps or "increase readability-grade" findings, none of which I can legitimately act on. I stopped here and did not write draft4.

## Choice

`final.md` = draft3. It has the fewest actionable High/Medium findings: both remaining highs are the "…" in exact labels, and every remaining medium is in the skip list. No fact changed, and it reads at least as clearly as draft1.

## Fact check: final.md vs draft1.md

- The code block is identical.
- Every quoted string (labels, messages, tour titles, check names) is identical: `grep -o '"…"'` gives the same sorted set.
- Every key and shortcut token (Ctrl/Cmd/Shift combos, Page Down, Menu key, Enter, Esc) occurs the same number of times. Only bold was removed: from key letters, from the Error/Warning level words and from the troubleshooting lead-ins.
- Rewordings checked for meaning, all equivalent:
  - "nothing sets its level" → "its level floats"
  - "is not reported" → "is exempt"
  - "only input pins, input ports and sheet entries are reported" → "reports input pins, input ports and sheet entries, and ignores every other kind of pin or port"
  - "there are no results" → "the check can't list any findings"
  - "No place to jump to" → "Clicking the row won't move the view" (true: a reveal without a spot leaves the frame, F70)
  - "has no wire at all" → "is left unwired"
  - "has no Waive… item" → "can't be waived: its menu lacks Waive…"
- No sentence shifted meaning, so nothing needed fixing back.
