# cad-howto-erc: verifier report (stage 2)

Checked against /home/user/cadatomic (`ca/`), /home/user/ecadabra (`cf/`) and, for the context
menu keys, /home/user/svelters (`sv/`; I read the local checkout, not the sha in `pins.toml`). I read
code only and built nothing.

## 1. Fact-sheet verdicts

| # | Verdict | Note (truth, with path:line) |
|---|---|---|
| F1 | **unsupported** (hint) | The label "Check (ERC)" is right (`ca/crates/cadatomic-ui/i18n/en.ftl:1254`; it is set at `ca/crates/cadatomic-app/src/board/sheet.rs:1440-1444`). The hint string `ecad-erc-check-hint` (en.ftl:1255) is defined but no code reads it (grep finds only the .ftl line), so a user never sees it. |
| F2 | ok | sheet.rs:1112-1126. The button is the second from the right, next to Follow. |
| F3 | ok | en.ftl:1248-1249; inventory.toml:523; pcb.rs:141 |
| F4 | ok | inventory.toml:1861-1862, notes at 1744 |
| F5 | ok | inventory.toml:4010-4013. No `key_bindings.item` targets `ecad.erc.*`. |
| F6 | ok | board/erc.rs:117-137; it is called from board/schematic.rs:977 after every sheet rebuild |
| F7 | ok | app.rs:3225-3234 (only when `asked`) |
| F8 | ok | en.ftl:745-751; app.rs:3221-3224. The status line is also updated after every automatic rerun. |
| F9 | ok | cadatomic-ecad/src/erc.rs:105-122 |
| F10 | ok | cadatomic-ecad/src/erc.rs:140 |
| F11 | ok | cf/crates/schematic/src/erc.rs:147-158 |
| F12 | ok (nuance) | erc.tour.ts:19-20; cf erc.rs:9-14. Two kinds are Copperfield's own (config.rs:84-108), and the matrix is "inferred" (F29). |
| F13 | ok | en.ftl:1256; rows.rs:82-99. The row's level word is "Info". |
| F14 | ok | en.ftl:1257; rows.rs:62-80 |
| F15 | ok | en.ftl:390, 364; chrome.rs:187-213 (`view_toggles` of the View menu) |
| F16 | ok | en.ftl:567; rows.rs:132-146 |
| F17 | ok | ProblemsPanel.svelte:131-137; rows.rs:121-131. A row with no place shows only the schematic's name. |
| F18 | ok | rows.rs:117-120; the panel never reads `detail` (ProblemsPanel.svelte:113-139) |
| F19 | ok | cadatomic-ecad/src/erc.rs:99, 184 |
| F20 | ok | board/erc.rs:286-316 |
| F21 | ok | board/erc.rs:218-264; cadatomic-ecad/src/erc.rs:29-31 (10.16 mm). The reveal runs on *select* too, not just a click (bottom.rs:191-199). |
| F22 | ok | ProblemsPanel.svelte:4-6; erc-waive.tour.ts:54-56; it is built in `sv/crates/skelter-widgets/src/views/menu.rs:503-504` |
| F23 | ok | ProblemsPanel.svelte:142-155; rows.rs:144 |
| F24 | ok | cadatomic-ecad/src/erc.rs:11-13 |
| F25 | ok | catalog.rs:876-886; checks.rs:312-339 |
| F26 | ok | checks.rs:326 |
| F27 | ok | catalog.rs:276-286; checks.rs:172-205 |
| F28 | ok | config.rs:139 (L17), 176-178. Sheet entries of all four kinds are W. |
| F29 | ok | config.rs:119-121 |
| F30 | ok | catalog.rs:288-298; checks.rs:341-395 |
| F31 | ok | catalog.rs:300-310; checks.rs:397-434 |
| F32 | **wrong (citation)** | The content is right, but L1 is config.rs:**123** and L3 is config.rs:**125** (not 124/129). L8 is :130. |
| F33 | ok | catalog.rs:312-322; checks.rs:84-107. I recomputed every cell: Output+Power E, Output+OC E, IO+Output W, Input+Output N. |
| F34 | ok | cf/crates/model/src/erc.rs:199-218 |
| F35 | ok | model/src/erc.rs:161-175 |
| F36 | ok | en.ftl:1677, 1683, 1777. The design keeps a *pinned copy* of each part, so an edit to the library reaches a placed part only through "Update from library" (en.ftl:1396-1397). See draft issue 2. |
| F37 | ok | catalog.rs:336-346; checks.rs:245-310 |
| F38 | ok | catalog.rs:852-862; checks.rs:436-486 |
| F39 | ok | catalog.rs:552-562; checks.rs:488-532 |
| F40 | ok | config.rs:84-96; checks.rs:534-577. It has no place: `cx.violation` is called without `.at()`. |
| F41 | ok | config.rs:97-108; en.ftl:1268 |
| F42 | ok | cadatomic-ecad/src/erc.rs:172-175 |
| F43 | ok | No ERC settings strings in en.ftl and no `ErcOptions` use in cadatomic. Copperfield reads levels from the store (config.rs:246-272), and an Altium import writes them (config.rs:6-7; cf/crates/io-altium/src/prj.rs). |
| F44 | ok | inventory.toml:2644, 2689, 2698, 2707, 2743, 2752. All are layer **Viewport**, so the pointer must be over the sheet. |
| F45 | ok | en.ftl:1034 |
| F46 | ok | erc.tour.ts:111-122 |
| F47 | ok | nets.rs:7-8, 246-250 |
| F48 | ok | en.ftl:1031, 981-982 |
| F49 | ok | power-port.tour.ts:8-14, 30-34, 44-49 |
| F50 | ok | junction.tour.ts:8-14; en.ftl:1032 |
| F51 | ok | net-label.tour.ts:11-13 |
| F52 | ok | en.ftl:960-961; annotate.tour.ts:12-13 ("the lowest number its letter has free") |
| F53 | **doubtful** | The tour selects the old text first (`press('Primary+A')`, properties.tour.ts:80) before typing, and says the pane "opens as a tab over the sheet" (properties.tour.ts:49-51). The fact leaves both out. |
| F54 | ok | cross-probe.tour.ts:51-60. The modifier is `primary`, which is Cmd on macOS. |
| F55 | ok | en.ftl:1071-1075; ProblemsPanel.svelte:73-86, 119-130 |
| F56 | ok | ProblemsPanel.svelte:77-80; board/erc.rs:67-69 |
| F57 | ok | cadatomic-ecad/src/erc.rs:225-249; board/erc.rs:12-17 |
| F58 | ok | board/waivers.rs:12-19 |
| F59 | ok | rows.rs:102-106; en.ftl:1954 |
| F60 | ok | board/erc.rs:272-277 |
| F61 | ok | en.ftl:1065; rows.rs:117-119, 177-183 |
| F62 | ok | rows.rs:115-116; cadatomic-ecad/src/erc.rs:236-239 |
| F63 | ok | nets.rs:147-158; cf erc.rs:15-19, 221-222 |
| F64 | ok | en.ftl:1072, 1950-1951; board/erc.rs:99-110 |
| F65 | **wrong (citation)** | erc-waive.tour.ts has only 122 lines. The undo step is at **:109-119**, not 234-244. The content is right. |
| F66 | ok | en.ftl:1939-1949 |
| F67 | ok | No `noerc`, `NoErc` or `Directive` in `cadatomic-ecad/src/sch*.rs`. Only edit.rs and erc.rs name them. |
| F68 | **wrong (citation)** | The waive tour's title is at erc-waive.tour.ts:**27**, not 152. en.ftl:1087 and erc.tour.ts:29 are right. |
| F69 | ok | cf/crates/cli/src/main.rs:17, 53-56, 830-933; exit 1 at :170. For a PrjPcb/SchDoc input it also prints the levels it read (:883-902). |

## 2. Draft issues

1. **"Drawing disagrees with the circuit | Error | … | Go to the place and redraw the connection."**
   Verdict: **wrong / step can't be done.** The check attaches no place to this finding
   (`cf/crates/schematic/src/erc/checks.rs:560-572` calls `cx.violation` with no `.at()`). Its row
   therefore shows only the schematic's name (rows.rs:125-131), puts no marker on the sheet, and
   clicking it changes nothing (`board/erc.rs:248`, `if let Some(sheet) = m.sheet()`). The draft
   also contradicts itself: the waiver section lists this finding among those with "no place on
   the sheet". Fix: "This finding has no place to jump to. Look for a wire end that lands on a pin
   of another net, or a drawn group that joins two nets, and redraw that connection."

2. **"fix the pin's **Electrical type** in the part's symbol in the library."**
   Verdict: **incomplete procedure.** The design keeps pinned copies of its parts. A library edit
   reaches the design only after "Update from library" ("Replace the design's pinned copies of
   these parts with their libraries' current versions…", en.ftl:1396-1397). Without that step the
   finding stays. Fix: add "…then run **Update from library** on the design so the placed part
   picks up the change."

3. **"Press **P** then **W**, click the end of the pin…"**, "(Page Down zooms out at the
   pointer)", "press **T** then **A**", "Press **P** then **V**", "Press **P** then **O**"
   Verdict: **steps a user may not be able to perform as written.** Every one of these keys is
   bound on the **Viewport** layer (inventory.toml:2644-2760), and Page Down is read by the sheet
   canvas (`board/sheet.rs:439-442`). The step before them is "Click the row" in Problems, which
   leaves the keyboard and pointer in the Problems list, where Page Down scrolls the list. The
   tours always say "Click the sheet" or "With the pointer on the sheet" first (erc.tour.ts:92-93,
   97, 106; power-port.tour.ts:40). Fix: start step 2 with "Click an empty spot of the sheet (or
   rest the pointer on it)", and say once in "Before you start" that the tool keys work with the
   pointer over the sheet.

4. **"If you have Copperfield's command-line tool, `copperfield erc` runs the same checks…"**
   (writer's doubt e) Verdict: **presented as available, but it doesn't ship with cadatomic.**
   cadatomic has no `cf-cli` dependency and no binary, script or doc that installs `copperfield`.
   The only mentions are in research notes (`ca/docs/research/notes/2026-10-25-…/internal_cadatomic_plans.md:170`).
   A user also won't know what "Copperfield" is. Fix: either cut the section, or say "The ECAD
   engine cadatomic uses, Copperfield, has its own command-line tool. It isn't installed with
   cadatomic; you build it from the Copperfield source." Worth adding: its output prints each
   finding's full sentence with net and pin names (main.rs:919-923), which the app's list doesn't
   show.

5. **"The row stays in Problems, now reading **Waived** … so the next person to read the design
   sees it."** and **"It records your reason, your name … and today's date."**
   Verdict: **misleading without the gap note (writer's doubt a, confirmed).** The panel draws
   only the level word, the check name and the place (ProblemsPanel.svelte:131-137). The reason,
   author and date go into `detail` (rows.rs:117-120), which the panel never reads, so nobody can
   see the reason in the app. (The waive tour says the row shows the reason, at
   erc-waive.tour.ts:75 and :83. That is the tour's mistake; don't copy it.) The draft says this
   only in "Gaps", well after the procedure. Fix: in "What happens", add "The reason is saved with
   the design, but this version of the app doesn't show it in the list."

6. **"…not the kernel's sentence with the net and pin names … Both are kept with the row's data,
   but the list does not draw them yet; select the row to see the place."**
   Verdict: **internal jargon** ("kernel", "the row's data", "draw"). Fix: "The list doesn't show
   the full message, which names the net and pins (for example "Net N0 has only one pin
   (R1-1)"), or a waiver's reason, author and date. Click the row to see the place on the sheet."

7. **"Unconnected objects in net | Set by the connection matrix"**
   Verdict: **incomplete.** Every column in the table gives a default level except this one. The
   default is **Warning** for the items listed (matrix line L17, `cf/…/erc/config.rs:139`, read
   at :176-178; catalog default Warning, catalog.rs:281). Fix: "Warning (from the connection
   matrix)".

8. **"The design could not be turned into a circuit, so there are no results. Undo your last
   change and run the check again."**
   Verdict: **unsupported advice.** Nothing in the code points to the last change as the cause.
   The failure can come from a design as it was opened or imported. Running the check again is
   also unnecessary, because the check reruns by itself after an undo (`board/erc.rs:133-137`).
   Fix: "If it appeared after an edit, undo that edit; the check reruns by itself."

9. **"hold Ctrl and click one of its pins. The whole net lights up, on the sheet and on the
   board."**
   Verdict: **platform gap and an overstatement.** The tour's modifier is `primary`
   (cross-probe.tour.ts:57), which is Cmd on macOS; the draft gives Cmd for the palette and undo
   but not here. "On the board" is true only when the design has an open board. Fix: "hold Ctrl
   (Cmd on macOS) … on the sheet and, if the design has a board open, on the board."

10. **"…or, if the input should sit at a fixed level, tie it to a supply: Press P then O…"**
    Verdict: **minor accuracy gap.** A power port is a non-pin item. If no other pin in the
    design is on that supply, the input becomes a net with one pin plus a naming object, and that
    reports **Nets with only one pin**, an Error (checks.rs:326). Fix: add "(use a supply name
    other parts are already on)".

11. **"Nets with no driving source | … The net has no pin that drives it (output, IO, open
    collector, open emitter, HiZ or power pin)."** and **"that warning clears only when a driving
    pin is on the net"**
    Verdict: **incomplete, and "only" overstates it.** Input and bidirectional ports and output
    and bidirectional sheet entries also count as drivers. A net whose items are all passive is
    never reported (checks.rs:341-345, 367-378). Fix: "no driver (an output, IO, open-collector,
    open-emitter, HiZ or power pin, or an input port or output sheet entry)", and drop "only".

12. **"This is the error you will see most."**
    Verdict: **unsupported.** No source says this. The tours' demo happens to be made of
    single-pin nets, but that isn't evidence. Fix: "This is a common one:" or drop the sentence.

13. **"select one, open the **Properties pane** from the command palette, double-click
    **Designator**, type a new one and press Enter."**
    Verdict: **doubtful step.** The tour selects the old text first (`Primary+A`,
    properties.tour.ts:80) before typing, so typing straight away may add to the old name instead
    of replacing it. The pane also opens as a tab over the sheet (properties.tour.ts:49-51).
    Fix: "…double-click **Designator**, select its text (Ctrl+A), type the new one and press Enter.
    The pane opens as a tab over the sheet; click the sheet's tab to go back."

14. **"Its checks and default levels follow Altium Designer's."**
    Verdict: **acceptable; softening optional (writer's doubt b).** Two checks are Copperfield's
    own ("Drawing disagrees with the circuit", "Sheet placed several times…", config.rs:84-108).
    The default matrix is the one 9 of 11 Altium example projects store; Altium's built-in default
    was never read (config.rs:119-121). Don't put "inferred" in user prose. If anything: "Most of
    its checks and their default levels come from Altium Designer's."

15. **"A design can carry its own levels (an Altium project does), and those win"**
    Verdict: **imprecise.** The levels come from importing a PrjPcb, which writes `erc_level` and
    `erc_matrix` lines (config.rs:4-7). Fix: "A design imported from an Altium project keeps
    that project's levels, and those win."

16. **"Each row … shows three things: the level …, the name of the check …, and where it is: the
    schematic, the sheet and the position in millimetres."**
    Verdict: **minor.** A finding with no place shows only the schematic's name (rows.rs:125-131).
    Fix: add "(only the schematic, for findings with no place)".

17. **"Once it has run, the check follows the schematic: every edit, undo and redo reruns it…"**
    Verdict: **minor gap.** The "on" flag lives in the open session (`SchematicErc.on`,
    board/erc.rs:33-35). It isn't saved with the design, so after the design is reopened the
    check has to be run again. Fix: add "until you close the design".

18. **Length.** The draft is 2,108 words including the table and the code block. The prose is
    close to the brief's 2,000-word ceiling. Trimming issues 6 and 12 and the CLI section (issue 4)
    would help.

### Writer's doubts: the answers

- **(a) Row detail and waiver reason:** confirmed not rendered (ProblemsPanel.svelte:113-139).
  The "Gaps" bullet is correct. The waive procedure needs the warning too (issue 5).
- **(b) "Inferred" matrix:** true in code (config.rs:119-121). The draft's wording is fine
  (issue 14).
- **(c) No ERC directive drawing and no ERC settings page:** both confirmed. No directive
  handling in `cadatomic-ecad/src/sch*.rs`; no ERC level or matrix strings in en.ftl; no
  `ErcOptions` use in cadatomic.
- **(d) Menu key / Shift+F10:** supported. skelter's `context-menu` opens on Menu or Shift+F10
  (`sv/crates/skelter-widgets/src/views/menu.rs:503-504`, local checkout). The menu's items
  follow the selected row (`current`, ProblemsPanel.svelte:66-68, 143). The draft's wording "on
  the selected row" is right.
- **(e) `copperfield` CLI for cadatomic users:** not shipped (issue 4).

### Lane voice and jargon

Issue 6 ("kernel's sentence", "row's data"), issue 4 ("Copperfield" unexplained), and issue 14
(don't add "inferred"). "Sheet pane" and "the strip" are mild; the app's own text says "sheet"
and the tool's strip is labelled "Name" (en.ftl:1035). I found no milestone ids, batch numbers
or `§` references in the draft.

### What checked out

These are verified and need no change: every catalog name in the table, every default level
except issue 7, the three-findings-from-one-unwired-input example (nets.rs:5-7 with checks.rs
172-205, 341-434), the no-connect exclusion, unwired passive pins going unreported, the ring and
cross colours, the status-line text, "no violations", every waiver label ("Waive…", "Why is it
acceptable?", "Waive", "Cancel", "Remove waiver", "Waived"), what the waiver records, waived
findings leaving the totals and the sheet, waiver matching, undo, Help ▸ **Tours…** and the two
tour titles, and the palette name and its shortcut.

## 3. Counts

- **Claims checked:** 69. **ok:** 64. **wrong:** 3 (all citation line numbers: F32, F65, F68;
  the content is right). **unsupported:** 1 (F1, the hint is never shown). **doubtful:** 1 (F53).
- **Draft issues:** 18.
  - wrong, or a step that can't be done: 2 (issues 1, 3)
  - incomplete procedure: 1 (issue 2)
  - presented as available but not part of the product: 1 (issue 4)
  - misleading, or a gap left until too late: 1 (issue 5)
  - jargon: 1 (issue 6)
  - unsupported claim or advice: 2 (issues 8, 12)
  - incomplete or imprecise fact: 7 (issues 7, 9, 10, 11, 15, 16, 17)
  - doubtful step: 1 (issue 13)
  - optional softening: 1 (issue 14)
  - length: 1 (issue 18)
  - exact UI label mismatches: 0
