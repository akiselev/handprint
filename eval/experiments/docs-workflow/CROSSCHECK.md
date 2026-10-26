# Cross-page check (DOCS.md 2c.17), 2026-10-26

Read at: cadatomic `b350c24` (svelters pin `6689112c`), svelters `6c1ea458` (master, S2 merged), ecadabra
checkout as found. Nothing built or run. Sources: cadatomic `docs/fix-queue.md` (Open, Fixed), `STATUS.md`
handoff, `docs/notes/rmegui/reports/{shell,ecad,viewport}.md`; svelters `STATUS.md` (rmegui, qc-a11y),
`plans/cloud/presto-b1/FIX-QUEUE.md`. Pre-edit copies: `<doc>/published.pre-crosscheck.md`.

## Ground truth established first

- **Default shell is skelter (native).** `crates/cadatomic-app/Cargo.toml:85` `default = ["dialogs",
  "tour-fixtures", "skelter"]`; `src/main.rs:36-41` runs `cadatomic_app::skelter::run` unless the build has
  `egui-shell` *and* `--egui` is passed. A default build has no egui route.
- **FQ-64 (native Problems body, no ERC/DRC rows, no row menu/Waive…) looks stale on master.** It was recorded
  on panes-ecad's head `4cba119`/`60377dc`, which lacks the shell lane's `c6760a4`. On master,
  `app.rs:2047-2063` (`panel_body`, REGION_BOTTOM, "CH3") mounts the **Svelte** Problems root whenever it draws
  (`svelte/panes.rs:225-232`: ui/ loaded, no fault) and falls back to `problems/native.rs` only otherwise. The
  shell report says so: `docs/notes/rmegui/reports/shell.md:37-38,118` "FQ-64 … fixed by `c6760a4`". The
  fix-queue row was never moved, and `board_drc`/`board_waivers` and the ERC tours have **not been re-run on
  merged master** (STATUS: "`check.sh --batch` and the tour count were not run"). So: listed in code,
  unverified. The Svelte panel has the Schematic section, row menu, **Waive…** and **Show waived**
  (`ui/panels/problems/ProblemsPanel.svelte:89-157`).
- **FQ-91 (Ctrl+Shift+P) and FQ-96 (typing into Svelte fields) were also recorded on a lane head without the
  shell lane** (viewport-tours `0021ad5` lacks `c6760a4`/`c43ae12`). Evidence they may not hold on master:
  `tests/skelter_a11y.rs:351-356` (shell lane, `c43ae12`) opens the Svelte palette with a raw Ctrl+Shift+P;
  `tests/palette_tree.rs:84-108` types into the Svelte palette's field with `type_text`; shell.md:57 says
  `board.components`/`nets`/`shell.files` stop at "the runner's `fill` into Svelte fields" (a tour-runner
  limit). Still open in the queue and unverified, so the pages keep hedged wording.
- **ERC waive route today:** only the Svelte Problems row menu → **Waive…** → inline `<text-edit>` reason
  (`ProblemsPanel.svelte:119-130,142-156`). `ecad.erc.waive` exists as a command
  (`board/schematic.rs:144`) but needs id, sheet, position, kinds, reason, author and date
  (`cadatomic-core/src/sch_edit.rs:485-496`): not a user route. No canvas route. `copperfield erc` reports
  waivers, doesn't create them. **Remove waiver** needs no typing.
- **svelters S2 fixes:** FQ-65 (`90061e54`), FQ-67, FQ-72, FQ-73..76 are on svelters master `6c1ea458`
  (`PropError::coded` at `skelter-core/src/error.rs:62`; `tests/app_values.rs` record props, `.code`), not at
  cadatomic's pin `6689112c` (no `coded`, no `app_values.rs`).

## cad-howto-erc (5 corrections)

| # | Problem (quote) | Source | Correction |
|---|---|---|---|
| 1 | "type the reason and press Enter" (waive workflow) — the reason field is a Svelte `<text-edit>`, the class of field the LED page's notice says doesn't take typing | FQ-96 (`fix-queue.md:97`); `ProblemsPanel.svelte:121-128` | Top notice: typing into some form fields doesn't reach them, the waiver reason field is that kind; checking and fixing still work, waiving may not. Troubleshooting "Waive… did nothing" points to it. Same wording family as the LED notice. |
| 2 | "From the command palette, Ctrl+Shift+P, or Cmd+Shift+P on macOS" | FQ-91 (`fix-queue.md:92`); LED log dropped the shortcut | Palette opened by clicking **Search commands…** in the title bar (`en.ftl:324`, `shell/native.rs:258-259`; click route tested in `palette_tree.rs:84-90`). Shortcut removed. |
| 3 | "open **Components library** from the command palette" / "open **Properties pane** from the command palette" — palette-only routes; the LED page uses View ▸ Open as tab ▸ Board properties for the same pane | FQ-91; `shell/chrome.rs:287-297` (Open as tab lists every non-pinned view); `board/lib_pane.rs:178`, `board/panels.rs:151` | Now **View ▸ Open as tab ▸ Components library** and **View ▸ Open as tab ▸ Board properties**, matching the LED page. |
| 4 | "The bottom area opens on **Problems**, at its **Schematic** section" — contradicts the LED page ("the Problems tab may not list the results"); true in master's code, but the fallback body has no ERC rows or row menu | FQ-64 (`fix-queue.md:87`, stale per shell.md:37-38); `app.rs:2055-2059`; `problems/native.rs:94-174` | Kept the steps (code supports them); added a troubleshooting item: if the sheet has marks but Problems lists none, the tab is showing a plainer list without the check's results or a row menu; go by the marks. Consistent with the LED page. |
| 5 | "Help ▸ **Tours…** has two guided tours" — `schematic.erc`, `schematic.erc-waive` fail on the native shell | FQ-61 (`fix-queue.md:71`), ecad.md:136 | Added "In current builds either may stop before the end." |

Not changed (noted): "select its text (Ctrl+A …)" in the Designator steps is redundant (the grid selects on open,
LED verify F68, `cadatomic-ui/src/skelter/property_grid.rs:545-549`) but harmless. **Update** reading disabled to
assistive tech after re-enable (FQ-67, a11y only; clicks work) — not a sighted-user problem.

**Main task on a default setup:** running the check and fixing findings: yes (button, sheet marks, edits; the
Problems list very likely, per code). Waiving: uncertain — depends on typing into a Svelte field (FQ-96 class),
unverified on master; no other route.

## cad-tutorial-led-board (0 corrections)

| # | Statement | Source | Verdict |
|---|---|---|---|
| 1 | Notice: New library **Name** typing doesn't reach it | FQ-96 | Kept. It is hedged ("if that happens to you"). Possibly a tour-runner limit rather than real typing (shell.md:57; palette typing works in tests), and recorded before the shell merge. |
| 2 | Step 6 "the **Problems** tab may not list the results" | FQ-64 | Kept (hedged); now matches the ERC page's troubleshooting item. Likely stale on master (see above). |
| 3 | "Naming with Tab may not take" | FQ-57, FQ-62 (`library.draw-symbol`) | Correct, open. |
| 4 | Menus "View ▸ Open as tab ▸ …" | FQ-59 tour half fixed (`fix-queue.md` Fixed) | Correct for the native shell. |
| 5 | Create / Create part / Export buttons | FQ-67 (a11y node stays disabled; clicks work) | No change: not a mouse-user problem. |
| 6 | "In the strip at the bottom, open the **Part** list" | FQ-26 (tour can't find `ecad.sch.tool.part`) | No change: a tour-locator row, no evidence the control is missing for a user. |

No palette shortcut on the page any more (log.md item 1). No contradiction left with the ERC page.

**Main task on a default setup:** step 1 certainly; steps 2–10 depend on the New library Name field (FQ-96
class). Unverified either way on master.

## sv-tutorial-first-skelter-app (1 correction)

| # | Problem (quote) | Source | Correction |
|---|---|---|---|
| 1 | "a project outside the svelters checkout cannot depend on these crates in the usual way. Work inside the checkout as this tutorial does." — reads as "outside is impossible"; the widget-kind how-to builds a crate beside the checkout with path dependencies | sv-howto-skelter/published.md:18-46; svelters `Cargo.toml:114-115,125-128` | Added: or give a crate beside the checkout path dependencies and copy its `[patch.crates-io]` and `[profile.dev.package.rquickjs-sys]` tables. |

Checked, fine: no `overflow` CSS (svelters F18); text-edit "does not draw itself yet" agrees with FQ-52 and was
observed; Windows adapter issue (FQ-90) is outside "run end to end only on Linux with X11". Executed under Xvfb at
`6c1ea458` (verify.md).

**Main task on a default setup:** yes (it was run).

## sv-howto-skelter (1 correction)

| # | Problem (quote) | Source | Correction |
|---|---|---|---|
| 1 | "A nested object … arrives in Rust as a nested `Value::Record`. A method that returns `PropError::coded(…)` throws an `Error` whose `.code` is …" — true on svelters master only; on cadatomic's pin record/any props throw, there is no `PropError::coded`, errors are strings | FQ-74, FQ-75 (`fix-queue.md:84-85`); svelters `skelter-core/src/error.rs:62` (master), absent at `6689112c` | Added one sentence: both need a recent svelters master; on older commits (including the one cadatomic builds against) the object write fails with `no descriptor for type 0`, `PropError::coded` doesn't exist, errors have no `.code`. |

FQ-73/74/75 against the example itself: no hit. Props are `F64`/`Str`; the method argument is a number
(`field.call('step', 1)`); `set_native` is called from `call` (an `EventCx`), not at mount; no `.code` relied on.
`SignalFlags::NONE` handler "receives the arguments alone" and "a SYNC signal's handler receives the event object
first" still hold after FQ-65's fix (`native-runtime/src/index.js:1283-1304,1393-1412`: only non-sync signals
called synchronously gained `SIGNAL_ARGS`). Tag `quantity-field` is free in `skelter.json`. FQ-41 (cadatomic's own
`<quantity-field>` draws no value) is a different kind; the page already says so.

**Main task on a default setup:** yes, on svelters master (not built or run by anyone).

## Counts

- Pages: 4. Corrections applied: **7** (ERC 5, LED 0, sv tutorial 1, sv how-to 1).
- Problems found from known bugs: ERC 4 (FQ-96, FQ-91, FQ-64, FQ-61), sv how-to 1 (FQ-74/75); cross-page
  contradictions: 3 (ERC↔LED Problems listing, ERC↔LED properties route, sv tutorial↔sv how-to dependency
  model).
- Fix-queue rows likely stale on master (recorded on lane heads without the shell lane's `c6760a4`): FQ-64
  (shell report says fixed), FQ-91, FQ-96 (evidence above). Recommendation: run `board_drc`, `board_waivers`,
  `TOUR=schematic.erc,schematic.erc-waive,library.make-led-symbols … every_tour_passes_on_skelter` and one
  hand test of typing into New library's Name on master; if they pass, drop the LED notice and the ERC page's
  notice and troubleshooting item.

## Live corrections (2026-10-26)

Source: `live/LIVE.md`, cadatomic `b350c24` built against the pinned siblings and driven with real
input under Xvfb. Live observation overrides the code reading and the fix-queue rows above (FQ-96 and
FQ-64 did not reproduce for a person typing; the row menu did fail).

### cad-tutorial-led-board/published.md (7 changes)

| # | Was | Live finding | Now |
|---|---|---|---|
| 1 | "Known blocker" notice at the top: Name stays empty, only step 1 can be finished | Typing `led-symbols` into Name, Tab, Accept created the library ("Library opened", Library tab, `library.ecad` on disk); Enter also works | Notice removed |
| 2 | Step 2: "If **Name** is still empty after you type, you have hit the blocker" | As 1 | Replaced with what you see: status "Library opened", a **Library** tab, Enter works as well as Accept |
| 3 | Before you start: two-key shortcuts "with the pointer over the canvas" | Hovering left the status at "P was pressed; waiting for the next key"; after one click on the sheet P then N worked | Click the canvas once first; hovering isn't enough |
| 4 | "With the pointer on the symbol / on the sheet, press P then P" (steps 2 and 5) | As 3 | "Click the symbol's canvas once / the empty sheet once, then press…" |
| 5 | Step 6: "Judge by these marks: … Problems may not list the results" | Problems listed the ERC row; amber marker and status-line count shown | Problems lists each finding as a row and the status line gives the count; a clean sheet shows "Electrical rules check: no violations" (from code, not run live) |
| 6 | Step 6: "Press Ctrl+S with the pointer on the sheet" | Focus rule from 3 (Ctrl+S itself not tested) | "Click an empty spot of the sheet, press Ctrl+S" |
| 7 | Step 8: Ctrl+Page Down on the board | Focus rule from 3 (board not tested) | Click an empty spot of the board view first |

Confirmed live, unchanged: New project… opens the OS folder dialog; File ▸ New ▸ New library opens
the form (Name, Folder, Accept, Cancel); New symbol + Prefix + Create opens the symbol; File ▸ New ▸
New board design creates `untitled-1` with **Check (ERC)** at the top right. The page names no
palette shortcut or tours. Not tested live: everything from pins onward (symbols, footprints, parts,
wiring, board, routing, export).

### cad-howto-erc/published.md (11 changes)

| # | Was | Live finding | Now |
|---|---|---|---|
| 1 | Top notice blamed the waiver reason field | Right-click, Shift+F10 and the Menu key open no row menu; no "Waive…" exists anywhere | Top notice: the row menu doesn't open, so Waive… and Remove waiver can't be reached; fixing still works |
| 2 | "Waive a finding you accept" presented as working | As 1 | Section kept, headed "Not reachable yet"; Menu key / Shift+F10 now "is meant to open" |
| 3 | "Remove a waiver, or undo one" presented as working | As 1 | Section kept, headed "Not reachable yet" |
| 4 | Tours pointer ("may stop before the end") | Help ▸ Tours… and the palette's ERC tour only set "No part is open; New part makes one" | Pointer dropped |
| 5 | Palette only via **Search commands…** | Ctrl+Shift+P opens the palette; the title-bar box shows the hint | "…or press Ctrl+Shift+P" restored |
| 6 | "Click a row. The sheet switches…" | Any mouse click highlights the section header; Down selects the row and frames the place ("1 object selected", view zoomed) | Select with the Down arrow; note that a click highlights the header. "Click the row" in the fix steps and "clicking their row" changed to select |
| 7 | Row format described abstractly | Observed row "Warning  Floating net labels  untitled-1 · Sheet1 · (0.0000, 5.0800) mm" | Example added |
| 8 | Two-key shortcuts "only with the pointer over the sheet" and four "With the pointer on the sheet" steps (wire, no-connect, power port, annotate) | As LED 3 | Click the sheet (an empty spot) first; hovering isn't enough |
| 9 | Troubleshooting: "Problems lists none of them … plainer list" | Problems lists the rows | Removed |
| 10 | Troubleshooting: "Waive… did nothing … reason was empty" | Waive… unreachable | Replaced by "No menu opens on a row" (no way round it); the empty-reason fact moved into the waive steps |
| 11 | (none) | Right-hand status counter stayed "0 errors, 0 warnings" after a 1-warning check | Troubleshooting item: go by Problems and the left status line; also items for the header-click and the second-key-does-nothing cases |

Confirmed live, unchanged: **Check (ERC)** at the top right; the bottom area opens on Problems at
its Schematic section with a "Show waived" checkbox; status line "Electrical rules check: N errors,
M warnings"; amber ring-and-cross marker for a warning; selecting a row zooms to and selects the
items. Not tested live: red error markers, other checks, background rerun on edit, Ctrl+click net
highlight, the waiver reason field, the `copperfield erc` CLI.

### Figures

`live/figures.json`: ERC page, `0023-25-after-erc.png` after "Run the check" (marker, Problems row,
status line); LED tutorial, `0011-13-after-accept.png` after "Make the symbol library" (the new
library's tab, "Library opened"). `0009-11-name-typed.png` was passed over: it shows a scratch path
and a stale "New library: fill in name" status.

Counts after live corrections: ERC 5 + 11, LED 0 + 7.
