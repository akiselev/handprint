# Log: cad-tutorial-led-board

## Pass 1 — verified fixes (draft0 → draft1)

Issues are numbered as in `verify.md` §2.

1. **H, Ctrl+Shift+P.** Fixed. The shortcut is gone. Every palette step now goes through a menu (Board properties, Pending changes, Save project) or a button (Check (ERC)), so the palette bullet went too. Palette typing works in native-shell tests (F107), but no step needs the palette any more.
2. **H, Name fields.** Fixed, with a change. A blocker notice now opens the page and names steps 2–3 and, through them, 4–10. Step 2 points back to it. The `--egui` way round the verifier offered is **rejected**: the ECAD tabs have only native bodies, and the egui `ui` body draws nothing (F93), so the board, sheet and library editors would be blank. New finding: **New board design has a real way through.** From the File ▸ New menu it shows no form and makes `untitled-1` (F96, F97; native-shell test `new_design.rs:176-191`). Only the palette route asks for a name. So step 5 now uses the menu, and step 10's output folder is `outputs/untitled-1/fab`. Draft0's "Name it `led` and click Accept" after the menu was wrong: no form appears there. No way round was found for New library: `name` is required with no default (F95), `ecad.lib.open` needs an existing folder, and FQ-96 records no workaround (F110).
3. **H, guided tour.** Fixed by removing the pointer to the tour. It fails on both shells (F109), and the brief says not to send readers to a failing tour.
4. **H, part J.** Fixed: "change **Name** to `J`". The part form's Name is a native field (F54), not one of the FQ-96 fields.
5. **H, Problems.** Fixed. Step 6 now says to judge by the marks and that Problems may not list the results of this check or the board's. Step 10 calls the board clean when there is no marker and no ratsnest line. Every claim that results are "listed in Problems" is gone.
6. **M, "sometimes".** Fixed: now "may not take in current builds".
7. **M, Pin table.** Fixed: added "It opens in its own **Pins** tab … then click the symbol's tab to carry on". The table is unverified on the default shell (F38). That is recorded in facts but not in the prose.
8. **M, Properties label.** Fixed: the pane is now **Board properties** everywhere. Added "If the pane isn't showing, choose View ▸ Open as tab ▸ Board properties" (F100).
9. **M, how to get the app.** Fixed: "There is no installer yet: build cadatomic from source and run `cadatomic`" (F108).
10. **M, board covered after Acknowledge.** Fixed: step 8 opens with "If **Pending changes** now covers the board, choose View ▸ Open as tab ▸ Board again" (F105).
11. **L, Pad kind before Hole.** Fixed in pads step 1 (F104).
12. **L, Page Down count.** Fixed: "until there is room for two more parts on the left and a wire below them" (F106).
13. **L, Tab before Accept.** Fixed: "press Tab, and click **Accept**" in both New library steps. Step 5 no longer has a form.
14. **L, header land overstated.** Fixed: the hole comes from Samtec, and the 1.7 mm land is "a safe size above the IPC-2221 minimum" (F103).
15. **L, Space direction.** Fixed: a sentence before the pin steps explains it (F101).
16. **L, cathode fab line.** Fixed. The how is now a sentence after the footprint table, and the table cell no longer carries it (F102).
17. **L, exact coordinates.** Fixed: the parts' drop points and the track corners now read "about …", with "Close is fine". The board corners stay exact (whole mm on the readout).
18. **L, jargon and length.** Fixed. "Pinned copy" is now "its own copy of a footprint", and **Up to date** is explained as "its copy matches `led-footprints`". Prose without the tables is 2,042 whitespace tokens, about 1,890 words not counting ▸, list numbers and the like (draft0: 2,116 tokens). To make room, these went: the palette bullet, `project.toml`, the 100 mil grid sentence (the coordinates carry it), the "passive" line, the 1.6 mm stack line, the radius steps and the "few milliamperes" remark. The export file list is shorter, and the body tools now sit in one paragraph.

Facts: the 16 ok* citations are corrected as the verifier gave them (spot-checked). F3 and F4 drifts are corrected. F10, F70 and F84 are marked stale on the default shell. F7, F28, F38 and F54 are extended. F92–F110 are added.

Can the tutorial be finished on a default install today? Probably not. Step 1 works. Step 2's New library Name field fails in every automated run, and the automated input is the same as typing (F94). Whether a person's typing also fails isn't recorded, so the notice says "if that happens to you". Steps 5–10 have no blocker of their own once libraries exist.

## Pass 1 — handprint (draft1 → draft2)

Before: `tokens=2207 tier=marginal findings_total=37 high=14 med=12 doc-style high=0 med=0`.
After: `tokens=2212 tier=marginal findings_total=36 high=14 med=12 doc-style high=0 med=0`.

Acted on:
- `md.bold_rate` / `punct.asterisk_rate` (51.7 → 34.4): keys are no longer bold (Space, Tab, Esc, Ctrl+S, the "P then X" chords and the like). Neither are the terms set in bold for emphasis (net, ratsnest, courtyards) or the list lead-ins ("Pads.", "Fab outline:", "Getting the app." and so on). UI labels stay bold.
- `punct.semicolon_rate` (8.6, over the band): five semicolons split into sentences or rejoined with "and" or "which". The finding cleared.
- Bullets: "Before you start" is now two short paragraphs, and step 1's two-item numbered list is now prose.
- `punct.comma_rate`: split the intro's colon-and-comma sentence in two.

Skipped:
- `punct.ellipsis_*`: every … is in an exact label ("New project…", "Courtyard from pads…", "Export fabrication outputs…").
- `punct.en_dash_rate`, `punct.pipe_rate`, `punct.number_rate`: table ranges, pipes and coordinate values, all frozen.
- `dev.headline.number_initial`: the numbered step headings are what the frozen top notice refers to ("steps 2 and 3", "4 to 10").
- `read.*`, `form.latinate_rate`, `biber.nominalization_rate`, `punct.word_len*` (increase): these say the page is simpler than the human average, which is fine.
- `biber.pron.demonstrative`, `read.passive_rate`, `syn.ly_adverb_rate` (increase): adding them would add padding, not clarity.

## Pass 2 — handprint (draft2 → draft3)

Before: `findings_total=36 high=14 med=12`. After: `tokens=2230 tier=marginal findings_total=36 high=14 med=12 doc-style high=0 med=0`.

Acted on:
- `md.numbered_rate` (20.3 → 17.9) and the list-heavy layout:
  - Step 7's three numbered actions are now one short paragraph.
  - Step 3's two setup items ("make a second library", "create each footprint") are now prose.
  - The R/J bullets in step 4 are now two sentences, which also removes the last semicolon.
- `punct.comma_rate` (82.3 → 78.5): split four click chains (pin K, the header's pad 1, D1's return wire, R1's value) into two sentences each.
- `biber.modal.possibility` (1.36 → 1.79): one natural "can": "you can press Backspace to take back the last piece".

Skipped:
- `punct.bullet_rate` (0.45): the only match left is the "·" inside the exact chip label "3 footprints off the board · Place them".
- `syn.fragment_rate`: the matches are headings and numbered step items, which are the shape of a tutorial.
- Everything skipped in pass 1, for the same reasons.

## No pass 3: churn guard

Pass 2's findings are the same set as pass 1's. Counts held at 14 high and 12 medium. What's left is labels and tables (ellipsis, en dash, pipes, numbers), heading numbers the notice depends on, and "increase" readability findings. Going further would mean breaking up step lists that readers follow click by click. That trades clarity for a statistic, so I stopped.

## Chosen version

`final.md` = **draft3** (pass 2). It has the lowest values in every family I could legitimately act on:
- bold 34.1 (draft1: 51.7)
- numbered 17.9 (21.3)
- comma 78.5 (81.1)
- semicolons back in band
- no structural bullets left

No fact changed, and nothing is less clear. Prose without the tables is about 1,920 words not counting ▸, list numbers and the like.

## final.md vs draft1.md

I diffed word by word with bold markers removed. The tables are byte-identical, the top notice is byte-identical, and the step order is unchanged. No label, shortcut, value, coordinate, file name or identifier changed. Wording changes that I checked for meaning:
- "Press Space twice and Tab, and name the pin `K`. Then click two squares right…": the same actions in the same order.
- "A text reads `${REF}` unless you press Tab and set **Text** to `${VALUE}` before you click": the same as draft1's "`${REF}` by default; for `${VALUE}`, press Tab and set **Text** before you click".
- Step 7 as prose: the same three actions in the same order, with the same results (lists parts and nets; footprints arrive off the board).
- "Ctrl below means Cmd on macOS" became "Ctrl means Cmd on macOS": same meaning.

I found no shift of meaning, so nothing needed fixing back.
