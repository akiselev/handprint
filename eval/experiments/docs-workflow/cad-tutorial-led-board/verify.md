# Verify: cad-tutorial-led-board (draft0.md)

Verifier, stage 2. Read at cadatomic `b350c24` (clean tree). Nothing built. Paths are relative to
`/home/user/cadatomic/`. "Default shell" means the native skelter shell, the default build since
`STATUS.md:16`. In the tour files, "tour" means the file the draft's step follows.

## 1. Claim verdicts (facts.md)

`ok*` means the content is right but the cited line range is wrong. The correct range is given.

| F | Verdict | Note |
|---|---|---|
| F1 | ok | `make-led-board.tour.ts:35-56` |
| F2 | ok | `:16-23` |
| F3 | ok | en.ftl:1087, :1105-1106; inventory menu `cmd:help.tours.open` at :3361 |
| F4 | ok | All ids are registered. Lines drift by 1–2 (e.g. `ecad.items.move` is `pcb_edit.rs:284`, `ecad.board.place_unplaced` `pcb_board.rs:263`) |
| F5 | ok | STATUS.md:100. Also FQ-61 (fix-queue:71): on the default shell this tour already fails at "the start page shows" |
| F6 | ok | fix-queue:68 (egui build). FQ-62 (fix-queue:72) records the same Tab-to-Properties failure on the default shell (`library.draw-symbol`) |
| F7 | ok | fix-queue:97 (FQ-96). Incomplete: the New board design form uses the same `field-name` (`make-led-circuit.tour.ts:5,93`) |
| F8 | ok | fix-queue:92 (FQ-91) |
| F9 | ok | inventory:1744 |
| F10 | stale/doubtful | inventory:1861 binds it, but per F8 it does nothing on the default shell |
| F11 | ok | native.rs:258-259; en.ftl:323-324 |
| F12 | ok | |
| F13 | ok | |
| F14 | ok | |
| F15 | ok | pane.rs:377-384 (board needs focus/hover; the hover fix is recorded in FQ-62's fix column) |
| F16 | ok | en.ftl:61-62; Properties.svelte:96-104 |
| F17 | ok | en.ftl:1199-1203 |
| F18 | ok | |
| F19 | ok | |
| F20 | ok | project.rs:46-52 |
| F21 | ok | |
| F22 | ok | inventory:3273-3303 |
| F23 | ok* | Correct range `make-led-symbols.tour.ts:91-122`. The file has 158 lines, so `:172-202` doesn't exist |
| F24 | ok | |
| F25 | ok* | `make-led-symbols.tour.ts:13-66` |
| F26 | ok | |
| F27 | ok | Also `library/tool.rs:5-6`: counter-clockwise, quarter turn |
| F28 | ok | The pane where "Next pin" shows is titled **Board properties** (en.ftl:625; `cadatomic-ui/src/panels.rs:126-131`) |
| F29 | ok | tool.rs:3, :23-24, :164-169 |
| F30 | ok | |
| F31 | ok* | `make-led-symbols.tour.ts:20-23, 39-42, 57-60` |
| F32 | ok | |
| F33 | ok | |
| F34 | ok | |
| F35 | ok | |
| F36 | ok* | `make-led-symbols.tour.ts:24-29, 43-47, 61-65` |
| F37 | ok* | `make-led-symbols.tour.ts:131-155` |
| F38 | ok | The table opens as its own dock tab "Pins" (`pin-table.tour.ts:36`, en.ftl:1591). That tour is red on the default shell before its name step (fix-queue:72) |
| F39 | ok | (the tour runs it from the palette) |
| F40 | ok | |
| F41 | ok | |
| F42 | ok | |
| F43 | ok | Hole can be edited only when Pad kind isn't smd (`footprint/props.rs:146`) |
| F44 | ok | The default is 0.025 (`footprint/mod.rs:207`, `tool.rs:68`); the steps wrap (`pane.rs:431-432`) |
| F45 | ok | |
| F46 | ok | |
| F47 | ok | |
| F48 | ok | The default pad shape is rect (`footprint/tool.rs:96`), so the first 1.7 × 1.7 pad is square |
| F49 | ok | |
| F50 | ok | |
| F51 | ok | |
| F52 | ok | |
| F53 | ok | |
| F54 | ok | **Name** defaults to the symbol's name (`library/tree.rs:582`) |
| F55 | ok | |
| F56 | ok | `no pad` is at en.ftl:1886 |
| F57 | ok | |
| F58 | ok | |
| F59 | ok | en.ftl:1890-1891 |
| F60 | ok* | `make-led-circuit.tour.ts:81-99` (the file has 249 lines) |
| F61 | ok* | `:103-114`. The strip is at the bottom of the sheet (`board/sheet.rs` native strip `bottom: 4px`; `place-part.tour.ts:58-61`) |
| F62 | ok* | `:15-18` |
| F63 | ok* | `:131-141` |
| F64 | ok* | `:142-152` |
| F65 | ok* | `:154-196` |
| F66 | ok* | `:169-195` |
| F67 | ok | |
| F68 | ok* | `:209-222`. The grid selects the text when it opens on a double-click, so typing replaces it (`cadatomic-ui/src/skelter/property_grid.rs:545-549`) |
| F69 | ok* | `:226-238`; en.ftl:1249, :1254 |
| F70 | stale/doubtful | True of the tour design, but on the default shell the Problems tab shows only core diagnostics. ERC/DRC rows don't appear (fix-queue:87 FQ-64; `problems/native.rs:1-5`) |
| F71 | ok | |
| F72 | ok* | `ui/tours/board/steps.ts:6-21` (the file has 22 lines) |
| F73 | ok | |
| F74 | ok | |
| F75 | ok | |
| F76 | ok | |
| F77 | ok | |
| F78 | ok | |
| F79 | ok | `place-footprints.tour.ts:8-10, 43-46`: "a column right of the outline" |
| F80 | ok | |
| F81 | ok | |
| F82 | ok | |
| F83 | ok | |
| F84 | stale/doubtful | The markers part is ok. "Listed in Problems" fails on the default shell (FQ-64, fix-queue:87) |
| F85 | ok | |
| F86 | ok | |
| F87 | ok | |
| F88 | ok | |
| F89 | ok | |
| F90 | ok* | `export-needs-outline.tour.ts:10-14, 47-53` (the file has 57 lines) |
| F91 | ok | |

## 2. Draft issues

Severity: **H** stops or misleads the reader; **M** confuses or costs time; **L** polish.

1. **H.** Draft: "Click **Search commands…** in the title bar (Ctrl+Shift+P also opens it)."
   **Wrong.** On the default shell, Primary+Shift+P doesn't open the palette; clicking the field
   does (fix-queue:92, FQ-91; the writer's own F8). **Fix:** delete the parenthesis, or say
   "Ctrl+Shift+P is meant to open it too, but in current builds only the click works."

2. **H.** Draft: "Typing into the **Name** field of some forms, such as **New library**, can also
   fail to register, and there is no way round that one yet."
   **Stale/incomplete.** FQ-96 (fix-queue:97) records `field-name` reading `""` after typing in
   every automated run on the default shell, not just sometimes. The same field is the Name in
   **New board design** (`make-led-circuit.tour.ts:5,93`), so the bug blocks step 2, step 3 and
   step 5, and with it the whole tutorial after step 1. "No way round" isn't quite true either: a
   build with the `egui-shell` feature runs the old shell with `cadatomic --egui`
   (`crates/cadatomic-app/src/main.rs:15, 37-40`; `Cargo.toml:102-104`). That shell is
   temporary, and FQ-57 was recorded on it. **Fix:** name all three forms. Say plainly that if
   the name doesn't take, you can't continue past that step yet. Optionally add one line for
   people building from source ("`cadatomic --egui`, in a build with the `egui-shell` feature,
   still has the old window, where these fields work").

3. **H.** Draft: "The same walkthrough is a guided tour in the app: **Help ▸ Tours…**, then
   **Make an LED board, from nothing**, with **Watch it** or **Try it**."
   **Missing caveat.** The automated run of exactly this tour is red. It stops at "Press Tab and
   name pin 1 A in Properties." (STATUS.md:100; fix-queue:68), and on the default shell it fails
   even earlier, at the start page (fix-queue:71, FQ-61). **Fix:** add "The tour currently stops
   at naming the LED's first pin; follow this page from there."

4. **H.** Draft: "`J` from symbol `Conn_01x02` and `PinHeader_1x02_P2.54mm`, value `Conn_01x02`."
   after "Leave **Name** as `LED`".
   **Missing step, wrong result.** The New part form's Name defaults to the symbol's name
   (`library/tree.rs:582`), so "the same way" makes a part called `Conn_01x02`. The tour types
   `J` (`make-led-parts.tour.ts:94`), and step 5 then tells the reader to choose `J`
   (`make-led-circuit.tour.ts:109-110`). **Fix:** "…and change **Name** to `J`."

5. **H.** Draft, step 6: "Anything it finds is marked on the sheet (red for errors, amber for
   warnings) and listed in Problems." Step 10: "A violation shows as a circle with a cross and a
   row in Problems."
   **Stale on the default shell.** Its Problems tab mounts the native body, which lists only the
   core diagnostics. Board DRC rows don't appear (it can show "No problems." under a tab reading
   "PROBLEMS 4"), and neither do the ERC rows, including the clean row "Electrical rules check: no
   violations" (fix-queue:87 FQ-64; `problems/native.rs:1-5`; the ERC rows are built only for the
   Svelte panel, `problems/rows.rs:60-100`; en.ftl:1256). The markers themselves are drawn
   (`drc-markers.tour.ts:31-35`, `erc.tour.ts:52-55`). **Fix:** tell readers to judge by the
   markers. "Clean" means no marker on the sheet or board and no ratsnest line left. Add: "In
   current builds the Problems tab may not list these checks."

6. **M.** Draft: "Naming a pin with Tab while you place it sometimes doesn't take".
   **Unsupported.** "Sometimes" has no source. The failure is reproducible in automation on both
   shells (fix-queue:68 egui; fix-queue:72 `library.draw-symbol` "Tab to Properties, as FQ-57").
   Whether a person typing by hand hits it isn't recorded. **Fix:** "may not take".

7. **M.** Draft: "Click **Pin table** above the symbol, type the name into that pin's **Name**
   cell and press Enter."
   **Real but incomplete.** It matches `pin-table.tour.ts:31-38, 83-92` (the button is on the
   symbol's strip, the cell commits on Enter). But the table opens in **its own tab, "Pins"**
   (`pin-table.tour.ts:36`; en.ftl:1591), so the reader has to switch back to the symbol to go on.
   And on the default shell the pin-table tour fails before its name step (fix-queue:72,
   "electrical type"), so nobody has checked the workaround there. **Fix:** add "It opens in its
   own tab; click the symbol's tab to carry on."

8. **M.** Draft: "Properties shows **Next pin**" / "Properties shows the symbol itself" /
   "Properties shows the next pad".
   **Label differs.** The pane's tab reads **Board properties** (en.ftl:625;
   `cadatomic-ui/src/panels.rs:126-131`), including in the symbol editor
   (`edit-symbol.tour.ts:49`). Also, a reader who used the Pin table workaround may not have this
   pane open when the draft says to set **Value** "with nothing selected". **Fix:** "Tab opens the
   **Board properties** pane (Properties, below)…", plus one line: "If it isn't showing, choose
   View ▸ Open as tab ▸ Board properties."

9. **M.** **Missing prerequisite.** The page never says how to get or start the app. The
   repository has no README and no installer; `main.rs:13-17` gives the usage (`cadatomic`). The
   brief asks for gaps to be stated. **Fix:** one sentence: "There is no installer yet: build
   cadatomic from source and run `cadatomic`."

10. **M.** Draft step 8 starts "Press **Ctrl+Page Down** to fit the view" right after
    Acknowledge all.
    **Step missing compared with the tour.** The tour brings the board to the front again first
    (`make-led-board.tour.ts:100`, `showBoard('board-again')`), because Pending changes can sit
    in front of it. **Fix:** "If Pending changes covers the board, choose View ▸ Open as tab ▸
    Board again."

11. **L.** Draft: "Set **Width (mm)** and **Height (mm)** (and **Pad kind**, **Hole** or
    **Shape** where the table says)".
    **Order matters.** **Hole** can be edited only once **Pad kind** isn't `smd`
    (`footprint/props.rs:146`). The tour sets kind, then width, height and hole
    (`make-led-footprints.tour.ts:261`). **Fix:** "For the header set **Pad kind** to `th` first;
    **Hole** takes a value only then."

12. **L.** Draft: "Press Page Down a few times to make room on the left."
    **Doubtful.** The tour presses it up to 8 times, until both the LED's place and the return
    wire's corner below it are in view (`make-led-circuit.tour.ts:116-123`). **Fix:** "…until there
    is room for two more parts on the left and for a wire below them."

13. **L.** Draft: "type `led-symbols` in **Name** and click **Accept**" (and the same for
    `led-footprints` and the design `led`).
    **Differs from the tour.** The tour presses Tab between typing and Accept
    (`make-led-symbols.tour.ts:116`; `make-led-circuit.tour.ts:93`). The fields commit on Enter or
    when focus leaves them (`ui/panels/properties/Row.svelte:4-6`). Accept stays clickable while
    the field is being edited (`Properties.svelte:100`), so it probably works without the Tab,
    but the tested path has it. **Fix:** "type …, press Tab, then click **Accept**."

14. **L.** Draft: "The sizes come from Kingbright's APT2012EC (LED), Vishay's CRCW (resistor) and
    Samtec's TSW (header) datasheets."
    **Overstated.** For the header, Samtec gives only the 1.02 mm hole. The 1.70 mm land is the
    tour author's own choice, above IPC-2221's minimum (`make-led-footprints.tour.ts:136-139`).
    **Fix:** "…and the hole from Samtec's TSW page; the header's 1.7 mm land is a safe choice above
    the IPC-2221 minimum."

15. **L.** Draft, pins: "Press **Space** twice so the pin points left." The table rows for `R`
    and `Conn_01x02` give only "left/right".
    **Missing context.** The page never says that a new pin tool starts pointing right (rot 0,
    `library/tool.rs:164-169`) and that each Space press turns a quarter turn counter-clockwise
    (`tool.rs:5-6`). Without that, the reader can't work out the presses for R (twice, then twice
    more) or the connector (twice). **Fix:** one sentence before the table.

16. **L.** Draft table: "plus a fab line across the body at x = −0.6 to mark the cathode".
    **Missing how.** The procedure never says how to draw it. The tour uses Line, Fab, two clicks
    at (−0.6, −0.625) and (−0.6, 0.625), then Esc twice (`make-led-footprints.tour.ts:213-219`).
    **Fix:** add that to the LED row or to step 3 of the list.

17. **L.** Draft step 8.6 and step 9.4: "R1 to (28.025, 0) …", "Click above D1 at (42.175, 4)".
    **Usability.** The tour drops on exact points. A person can't place to three decimals by eye,
    and the corners don't have to be exact. **Suggestion:** "about here, using the X/Y readout;
    close is fine as long as courtyards don't touch."

18. **L. Jargon/length.** "a pinned copy of a footprint" (step 4) is app jargon the page doesn't
    explain. Something like "the part keeps its own copy of the footprint, and Up to date says
    whether it still matches the library" would do. The prose runs to about 2,100 words without
    the tables, over the brief's 2,000. I found no lane voice: no ids, batch numbers, `§`
    references or internal paths in the prose.

### Values checked against the tours (exact)

- **Symbol pins and bodies:** every pin (number, name, place, direction) and every body point
  in the symbol table matches `make-led-symbols.tour.ts:13-66`. Rot 180 is "Left"
  (`library/pins.rs:24`).
- **Footprints:** every pad size, pad centre, fab box, cathode line, silk box, designator point
  and grid step matches `make-led-footprints.tour.ts:192-293`.
- **Parts:** the LED map (pin 1 to pad 2, pin 2 to pad 1) matches `make-led-parts.tour.ts:65-72`.
  "The reverse" on pin 2's row comes out right.
- **Schematic:** the layout (6 × 2.54 mm), the wiring order and the three nets match
  `make-led-circuit.tour.ts:15-18, 169-195`.
- **Board:** the corners (26, −6)/(44, 6), the pad-1 drops 28.025/33.73/39.825, the corners
  (42.175, 4)/(28.025, 4) and the route order match `board/make-led-board.tour.ts:19-23,
  200-258`. The geometry is consistent: R1 pad 1 −0.975, J1 pad 1 −1.27 and D1 pad 1 −1.175 from
  origins 29, 35 and 41.
- **Labels:** every label the draft quotes exists in en.ftl. The one exception is the pane the
  draft calls "Properties", whose tab reads "Board properties" (issue 8).

### The writer's doubts, answered

- **The failing automated run and the Pin-table workaround:** the failure is real (STATUS.md:100,
  fix-queue:68, fix-queue:72). The workaround exists in the product, but nobody has verified it
  on the default shell (issue 7). The draft should also warn about the tour itself (issue 3).
- **Typing into form fields (fix-queue:97):** real. It hits New library twice and New board
  design (issue 2).
- **Ctrl+Shift+P / Search commands…:** the draft contradicts F8. Ctrl+Shift+P doesn't work on the
  default shell; clicking the field does (issue 1).
- **The ERC/DRC "no violations" text (fix-queue:87):** the draft quotes no such text, which is
  right. "Electrical rules check: no violations" (en.ftl:1256) is a Problems row that the default
  shell doesn't show, so tell readers to look at the markers instead (issue 5).
- **"Known rough edges" phrasing:** it's user-facing, with no FQ ids, but it's incomplete and
  softer than the evidence (issues 1–3, 5–6). Suggested shape: a short list of four plain lines
  (palette shortcut, Name fields, pin naming with Tab, Problems not listing checks), each with
  what to do instead or "no way round yet".

## 3. Counts

- **Facts:** 91 checked. 88 ok (16 of them with a wrong line citation, marked ok*), 3
  stale/doubtful (F10, F70, F84), 0 wrong, 0 unsupported.
- **Draft issues:** 18. By severity: 5 high, 5 medium, 8 low. By kind:
  - wrong or stale: 4 (1, 2, 5, 8);
  - missing step or caveat: 7 (3, 4, 9, 10, 11, 15, 16);
  - unsupported or overstated: 2 (6, 14);
  - differs from the tour: 2 (12, 13);
  - real but incomplete: 1 (7);
  - suggestion or style: 2 (17, 18).
- **Table values:** symbols, footprints, part map and board coordinates all match the tours
  exactly. No value errors.
