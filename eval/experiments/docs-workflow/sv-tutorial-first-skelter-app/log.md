# Revision log: sv-tutorial-first-skelter-app

## Pass 1 — verified fixes (draft0 → draft1)

Ground truth: verify.md's execution log (the tutorial ran end to end under Xvfb on lavapipe).

1. libxkbcommon-x11 prerequisite — **fixed.** Linux prerequisite now names libxkbcommon (+ -x11) and a Vulkan
   driver with the Debian/Ubuntu `apt install` line; dropped "links it"; new paragraph on winit loading
   Wayland/X11 at run time (Wayland if `WAYLAND_DISPLAY`, else X11) and on platforms (Vulkan→GL / Metal / DX12,
   CI on all three, page run only on Linux X11). Panic message added to troubleshooting. (F41 rev, F64, F65, F79)
2. Truncated compile errors — **fixed.** Troubleshooting shows the real `Load: Error loading module …
   error[unknown_element] …` shape with the `help:` line, plus a paragraph on the ~256-character cut and the
   shorter-path workaround. (F17 rev, F73)
3. Cache claim across restarts — **fixed.** Clause dropped; now "each start compiles the app again", fast for a
   component this size. (F19 rev, F76)
4. "cannot open window" cause/exit — **fixed.** Separate entries for no display (event-loop message, exit 1) and
   no GPU driver (`cannot open window #1: Failed to create surface …`, exits at once with status 0). (F33 rev, F68)
5. Software GL black window — **fixed.** "Chosen last" claim removed; lavapipe named as the working software
   driver; black-window-on-GL entry added to troubleshooting. (F42 rev, F66, F67)
6. Step 3 window look — **fixed.** Now 360×200, black background, light grey text at the top-left, "the black
   is the default theme". Step 4 window contents also stated from the shots. (F69, F70)
7. First log line timing — **fixed.** "one line as the app starts, before the window opens, and one per click";
   sample output kept (it was observed verbatim). (F71, C8 rev)
8. counter-egui `text-edit` draws nothing — **fixed.** Caveat added; also says how to run the fixture. (F78)
9. "Window path is new" bullet — **fixed.** Shortened to "No automated test of the windowed runner".
10. Lane voice ("cadatomic hosts its panels this way") — **fixed.** Sentence removed.
11. Build time — **fixed.** "a few hundred crates … under three minutes and about 2.4 GB" on four cores with
    `CARGO_PROFILE_DEV_DEBUG=0`; dropped "large workspace". (F74)
12. pnpm esbuild warning — **fixed.** One sentence after Step 1's command. (F75)
13. "nearest package.json" wording — **fixed.** Now: mounts `src/App.svelte` inside the given directory; the
    nearest `package.json` marks the app root.
14. `toolkit` field cosmetic — **rejected (no change needed).** Verifier rated it ok; Step 5 already says the
    runner/host overrides it with `cfg.backend = "skelter"`.

Also: Step 5 notes the `Cargo.lock` change and that the result matched (F77). "No reload in the runner" bullet
removed from the gaps list (Step 4 already says it) to stay within the 2,000-word limit. Fact sheet: line fixes
for F30, F32, F33, F51, F58; F61 source corrected; F22 count corrected; F41 marked doubtful; F64–F80 added.
Prose length: ~1,990 words (counted outside code blocks, headings included).

## Pass 1 — handprint (draft1 → draft2)

Before: `tokens=2048 p_same=0.107 findings_total=9 high=3 med=5 doc-style high=0 med=0`
After:  `tokens=2042 p_same=0.135 findings_total=7 high=0 med=5 doc-style high=0 med=0`

Acted on:
- punct.style.ellipsis_char_share / ellipsis_char_rate (high): the two `…` were my elisions, not real output.
  The no-display entry now quotes the message's real start and end; `[svelters] …` → "with the prefix `[svelters]`".
- biber.synthetic_negation_rate (high, 22 spans): rephrased the prose negations: "Nothing is bundled" → "rather
  than bundled"; "There is no published release" → "All three are unpublished"; "never runs in Node" → "runs in
  the Rust process"; merged the three-negation winit paragraph; "does not need a node_modules" → "can do without";
  "with no error code" → "lose the error code"; "with no text in it … nothing is printed" → "the label never
  appears … the terminal stays silent"; "takes no noticeable time" → "is quick"; "does not need to name" → "gets
  it without naming"; heading "What is not there yet" → "What is missing so far"; text-edit "does not draw
  itself yet: no box…" → "draws no box, text or caret yet".
- Bold-for-emphasis: "**If it fails to start.**" lead-in → plain sentence.
Skipped:
- Negations inside quoted error messages and the facts they state (`not installed`, `not an element`, `neither …
  nor`, "None of these tags are HTML", "not a rendering failure", "does not watch your files"): frozen or the point.
- read.gunning_fog / read.smog "increase": never make prose harder.
- biber.nominalization_rate, syn.ly_adverb_rate, biber.modal.possibility, form.latinate_rate "increase": would
  mean padding a plain page with abstract nouns/adverbs; no clarity gain.
- Self-check for the five tells: no template headings, list-closers, intensifiers, "It's <adj> to" openers or
  restating close found.

## Pass 2 — handprint (draft2 → draft3)

Before: `findings_total=7 high=0 med=5` (negation 7.84)
After:  `tokens=2037 p_same=0.135 findings_total=7 high=0 med=5 doc-style high=0 med=0` (negation 6.87, band ≤6.32)

Acted on: two more prose negations ("works when there is no GPU" → "works without a GPU"; "the binary links none
of them and a missing library shows up only at run time" → "a missing library shows up at run time rather than
at build time"). Skipped: everything else as in Pass 1. The remaining negation is almost all quoted error text.

Stopped after Pass 2 (churn guard): what remains is "increase" findings and frozen error text; a third pass
would chase the report.

## Chosen version

`final.md` = draft3 (Pass 2): no High findings, the lowest negation rate, and every remaining Medium is an
"increase" finding I chose not to act on. Prose ~1,977 words.

Diff check, final.md vs draft1.md: every fenced code block is byte-identical; no label, value, command, exit status
or quoted message changed. Meaning shifts reviewed: (1) the no-display entry quotes start and end of the message
instead of eliding the middle with `…`. It is the same message (F68). (2) "The binary links none of the display or GPU
libraries" is now implied by "loads … when the program starts" rather than stated (F41 rev, F65). (3) "takes no
noticeable time" → "is quick" (F76). None changes a fact.
