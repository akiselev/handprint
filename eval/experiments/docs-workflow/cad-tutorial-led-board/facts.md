# Facts: cad-tutorial-led-board

Paths are relative to `/home/user/cadatomic/` unless they start with `ecadabra/`
(`/home/user/ecadabra/`). Tour files are under `ui/tours/`. Read at cadatomic `b350c24`.

## The flow as a whole

- F1. The whole flow is one tour, "Make an LED board, from nothing": an LED, its series resistor and a two-pin connector, from an empty workspace to fabrication files, all drawn by hand; estimate 60 min. `ui/tours/project/make-led-board.tour.ts:35-56`
- F2. The flow's parts, in order: Start a project, the symbols, the footprints, the parts, the circuit, the board. `ui/tours/project/make-led-board.tour.ts:16-23`
- F3. Help ▸ Tours… opens the tours; a tour offers "Watch it" and "Try it". `crates/cadatomic-ui/i18n/en.ftl:1087`, `:1105-1106`; `docs/behavior/inventory.toml:3361` (menu entry `cmd:help.tours.open`)
- F4. Every command the flow uses is registered (inventory id and source): project.new `crates/cadatomic-core/src/project.rs:472`; ecad.lib.new `crates/cadatomic-core/src/pcb_lib.rs:134`; ecad.lib.symbol.create `pcb_lib.rs:650`; ecad.lib.symbol.pin.add `pcb_lib.rs:677`; ecad.lib.symbol.set `pcb_lib.rs:662`; ecad.lib.footprint.create/pad.add/graphic.add/courtyard.set/set_origin `crates/cadatomic-core/src/lib_footprint.rs:440,452,486,534,553`; ecad.lib.part.create/map.set `crates/cadatomic-core/src/lib_part.rs:198,224`; ecad.design.new `crates/cadatomic-core/src/pcb_board.rs:59`; ecad.component.place `crates/cadatomic-core/src/ecad_lib.rs:71`; ecad.sch.annotate/wire/sheet.tool `crates/cadatomic-core/src/sch_edit.rs:416,351,567`; ecad.erc.run, ecad.view.pending, ecad.view.fit, ecad.view.fabrication, ecad.export.fabrication `crates/cadatomic-core/src/pcb.rs:141,130,111,148,147`; ecad.pending.acknowledge `sch_edit.rs:402`; ecad.board.outline/place_unplaced `pcb_board.rs:214,263`; ecad.items.move `crates/cadatomic-core/src/pcb_edit.rs:284`; ecad.track.route `crates/cadatomic-core/src/pcb_route.rs:192`. (All listed in `docs/behavior/inventory.toml` at lines 415–1513.)
- F5. Known state: the automated run of this whole tour is currently failing ("Known red: FQ-57 (acceptance tour `project.make-led-board`)"). `STATUS.md:100`
- F6. The failure is at naming pin 1 "A" with Tab in Properties: the name does not take ("symbol.next.name=A is Ok(0)"). `docs/fix-queue.md:68`
- F7. On the native window shell (the default now: "skelter is the default shell", `STATUS.md:16`), typing into the task-panel form fields reaches nothing: `field-name` and `field-path` read `""` after the tour types (`docs/fix-queue.md:97`, FQ-96). `field-name` is the Name of New library (`ui/tours/library/make-led-symbols.tour.ts:9`, `:111-119`) and of New board design run from the palette (`ui/tours/schematic/make-led-circuit.tour.ts:5`, `:87-99`).
- F8. On the native shell, Ctrl+Shift+P does not open the palette; clicking the search field does. `docs/fix-queue.md:92`

## Keys and general UI

- F9. "Primary" is Cmd on macOS and Ctrl elsewhere. `docs/behavior/inventory.toml:1744`
- F10. Command palette: Primary+Shift+P is bound (`docs/behavior/inventory.toml:1861`). **Stage 3a:** on the default (native) shell the chord does nothing (F8, `docs/fix-queue.md:92`), so the draft no longer gives it.
- F11. The title bar has a search field, "Search commands" / "Search commands…". `crates/cadatomic-app/src/shell/native.rs:258-259`; `en.ftl:323-324`
- F12. Save: Primary+S (`file.save`, label "Save"). `inventory.toml:1933`; `en.ftl:262`
- F13. Undo: Primary+Z. `inventory.toml:1888`
- F14. Two-key shortcuts are "P then P" etc.: press one key then the next, with the pointer on the canvas. `ui/tours/schematic/place-part.tour.ts:58-61`
- F15. Page Down zooms out at the pointer, Page Up zooms in (board and sheet canvases). `crates/cadatomic-app/src/board/pane.rs:377-384`; `crates/cadatomic-app/src/board/sheet.rs:439`; `ui/tours/schematic/place-part.tour.ts:43-45`
- F16. The task panel's buttons are "Accept" and "Cancel". `en.ftl:61-62`; `ui/panels/properties/Properties.svelte:96-104`

## Project

- F17. Start page labels: "New project…", "Open project…", "New part"; summary "Start a project, open one, or make a part on its own." `en.ftl:1199-1203`
- F18. File ▸ New ▸ Project… also makes a project. `en.ftl:1170-1172`; `ui/tours/project/new.tour.ts:44-45`
- F19. New project asks for a folder; a new or empty folder becomes the project, named after the folder. `ui/tours/project/new.tour.ts:53-63`
- F20. A project folder holds `project.toml`, `designs/`, `libraries/`, `outputs/`. `crates/cadatomic-core/src/project.rs:47-53`; `ui/tours/project/new.tour.ts:33-36`
- F21. "Save project" writes the project's file and saves its changed documents. `en.ftl:281`, `:287`
- F22. File menu holds Save project and "Export fabrication outputs…". `inventory.toml` menus section (~3270–3300); `en.ftl:395`

## Symbols

- F23. File ▸ New ▸ New library; in a project it goes into `libraries/` and the project lists it. `ui/tours/library/make-led-symbols.tour.ts:91-122`; `en.ftl:1637-1638`
- F24. The library tree's new-symbol fields: "New symbol", "Prefix", button "Create". `en.ftl:1609-1611`; `ui/tours/library/steps.ts:55-63`
- F25. Symbols: LED (prefix D), R (prefix R), Conn_01x02 (prefix J). `ui/tours/library/make-led-symbols.tour.ts:13-66`
- F26. Pin tool: P then P; first pin is number 1; numbers count up. `inventory.toml:2536`; `ui/tours/library/steps.ts:72-78`
- F27. Space turns the next pin a quarter turn. `ui/tours/library/steps.ts:85-91`; `inventory.toml` symbol rotate (2599)
- F28. Tab while placing edits the next pin in Properties ("Next pin"); its row "Name". The pane's tab reads "Board properties" (`en.ftl:625`; `crates/cadatomic-ui/src/panels.rs:126-131`; `crates/cadatomic-app/src/board/panels.rs:150-151`). `ui/tours/library/steps.ts:96-103`; `en.ftl:1804`, `:1807`
- F29. New pins are passive and 100 mil long; pins snap to a 100 mil grid. `crates/cadatomic-app/src/library/tool.rs:3`, `:23-24`, `:168-169`
- F30. The symbol's reference point is a small cross at the origin. `crates/cadatomic-app/src/library/symbol.rs:480`
- F31. LED pins: 1 "A" at (−200, 0) mil pointing left (180°), 2 "K" at (200, 0) pointing right; R pins 1/2 at the same places; connector pins 1 at (−200, 0) and 2 at (−200, −100), both pointing left. `ui/tours/library/make-led-symbols.tour.ts:20-23`, `:39-42`, `:57-60`
- F32. Esc ends the pin tool. `ui/tours/library/steps.ts:124-131`
- F33. With nothing selected, Properties shows the symbol; set its "Value". `ui/tours/library/steps.ts:137-147`; `en.ftl:1819`
- F34. Body tools: P Y polygon (click corners, Enter), P L line, P R rectangle, P T text; Esc ends. `ui/tours/library/steps.ts:51-52`, `:159-185`; `inventory.toml:2545,2554,2581,2590`
- F35. A text tool's text defaults to `${REF}`; for `${VALUE}` press Tab and set "Text". `ui/tours/library/steps.ts:186-199`; `en.ftl:1814`
- F36. Body geometry: LED triangle (−100,100),(−100,−100),(100,0), bar (100,100)–(100,−100), ${REF} at (−100,150), ${VALUE} at (−100,−200); R rectangle (−100,−50)–(100,50), texts at (−100,100), (−100,−150); connector rectangle (−100,50)–(0,−150), texts (−100,100), (−100,−250). `ui/tours/library/make-led-symbols.tour.ts:24-29`, `:43-47`, `:61-65`
- F37. Save with Primary+S, then File ▸ Save project. `ui/tours/library/make-led-symbols.tour.ts:131-155`
- F38. Pin table ("Pin table" above the symbol) edits pin names in place; it opens as its own tab "Pins" (`ui/tours/library/pin-table.tour.ts:36`; `en.ftl:1591`). Not verified on the default shell: that tour fails earlier there (`docs/fix-queue.md:72`). `ui/tours/library/pin-table.tour.ts:31-38`, `:83-92`; `en.ftl:1758`, `:1770-1771`

## Footprints

- F39. Second library `led-footprints`, from New library. `ui/tours/library/make-led-footprints.tour.ts:173-189`
- F40. Library tree: type the name in "New footprint", click "Create footprint"; it opens empty. `en.ftl:1717-1718`; `make-led-footprints.tour.ts:61-68`
- F41. Footprint toolbar labels: "Pad", "Line", "Text", "Fab", "Silkscreen", "Origin: centre", "Courtyard from pads…", "Accept courtyard", "Grid {grid} mm". `en.ftl:1550-1551`, `:1558-1586`
- F42. Pad tool: click Pad, Tab shows the next pad in Properties. `make-led-footprints.tour.ts:72-80`; `en.ftl:1559`
- F43. Pad rows "Width (mm)", "Height (mm)", "Pad kind", "Shape", "Hole". `crates/cadatomic-app/src/footprint/props.rs:78-81`; `en.ftl:701-702`, `:1720-1723`
- F44. Grid steps 0.025, 0.05, 0.1, 0.635, 1.27 mm. `en.ftl:1587`
- F45. The footprint canvas shows the cursor as "X … mm, Y … mm" and "Next pad: …". `en.ftl:1542-1543`
- F46. LED_0805: pads 1.25 × 1.1 mm, centres at ±1.175 mm, pad 1 (left) the cathode (Kingbright APT2012EC); fab box ±1 × ±0.625 mm; cathode fab line at x = −0.6; designator at (0, 1.6). `make-led-footprints.tour.ts:132-133`, `:192-222`
- F47. R_0805: pads 0.95 × 1.45 mm at ±0.975 mm (Vishay CRCW); same fab box; designator at (0, 1.6). `make-led-footprints.tour.ts:134-135`, `:224-245`
- F48. PinHeader_1x02_P2.54mm: grid 1.27 (four clicks); pad kind th, 1.7 × 1.7 mm, hole 1.02 mm; pad 1 at origin square, pad 2 circle at 2.54 mm; after centring pad 1 is at −1.27; fab box ±2.54 × ±1.27; grid back to 0.025, silk box ±2.625 × ±1.325; designator at (0, 2.4). `make-led-footprints.tour.ts:136-139`, `:247-293`
- F49. Origin: centre puts the reference point at the pads' centre (where pick-and-place expects it). `make-led-footprints.tour.ts:116-124`; `en.ftl:1582-1583`
- F50. Courtyard from pads… shows the courtyard 0.25 mm round pads and body (IPC-7351 nominal); accept it. `make-led-footprints.tour.ts:105-113`; `en.ftl:1573-1578`
- F51. Line tool: Esc ends the line, Esc again the tool. `en.ftl:1561`
- F52. Designator text reads REF until on a board. `make-led-footprints.tour.ts:100`; `en.ftl:1567`

## Parts

- F53. Parts LED, R, J live in led-symbols, each with a pinned copy of its footprint. `ui/tours/library/make-led-parts.tour.ts:22-28`
- F54. Select a symbol in the tree; form "New part from {symbol}" with "Footprint library", "Footprint" ("Pick a footprint"), "Name", "Create part"; may need scrolling. Name defaults to the symbol's name (`crates/cadatomic-app/src/library/tree.rs:582`) and is a native field (`tree.rs:615-619`), not a Svelte form field; the tour types `J` for the connector's part (`ui/tours/library/make-led-parts.tour.ts:94`). `en.ftl:1907-1911`; `crates/cadatomic-app/src/library/tree.rs:567-625`; `make-led-parts.tour.ts:14-20`
- F55. A new part maps each pin to the pads of its number. `en.ftl:1824` (cmd-ecad-lib-part-create-tooltip, line 1824 region: "each pin maps to the pads of its number")
- F56. The part's pin rows have one toggle button per pad (pad name), "no pad" when unmapped. `crates/cadatomic-app/src/library/part.rs:1210-1228`; `en.ftl:1887`
- F57. LED map: pin 1 (A) to pad 2, pin 2 (K) to pad 1, because pad 1 is the cathode. `make-led-parts.tour.ts:61-72`
- F58. Values: LED, R, Conn_01x02. `make-led-parts.tour.ts:54-107`
- F59. Footprint status "Up to date" / "Out of date". `en.ftl:1891-1892`; `make-led-parts.tour.ts:111-126`

## Schematic

- F60. "New board design" (File ▸ New, or palette) asks for a name; in a project goes in designs/; the empty sheet opens. `en.ftl:1143-1144`; `inventory.toml` menu `menu-file-new` `cmd:ecad.design.new`; `ui/tours/schematic/make-led-circuit.tour.ts:81-99`
- F61. P then P starts the part tool; the strip's "Part" list offers the project's libraries' parts; Esc or "End tool" ends it. `make-led-circuit.tour.ts:103-114`; `en.ftl:1016`, `:1037`; `inventory.toml:2680`
- F62. Layout: J at (0,0), R six grid squares left, LED six more, the grid is 2.54 mm. `make-led-circuit.tour.ts:15-18`
- F63. Space twice turns the LED half round so its anode faces the resistor. `make-led-circuit.tour.ts:131-141`
- F64. T then A annotates: J1, R1, D1 ("Annotate schematic"). `make-led-circuit.tour.ts:142-152`; `inventory.toml:2644`; `en.ftl:960-961`
- F65. P then W starts wires; click pin ends, Enter ends a wire; clicks on empty sheet make corners; Esc ends the tool. `make-led-circuit.tour.ts:154-196`; `inventory.toml:2689`; `ui/tours/schematic/wire.tour.ts:61-63`
- F66. Wires: J1.1–R1.2, R1.1–D1.1, D1.2 round below back to J1.2; three nets. `make-led-circuit.tour.ts:169-195`
- F67. 330 Ω from 5 V: 5 V − 2 V LED = 3 V, about 9 mA. `ui/tours/concepts.ts:874-877`
- F68. Set value: click R1, palette "Properties pane", double-click Value, type, Enter. `make-led-circuit.tour.ts:209-222`; `en.ftl:875`, `:710`
- F69. ERC: "Check (ERC)" at the top right of the sheet, or "Check the schematic (ERC)" in the palette; finds nothing here; it rechecks after every change. `make-led-circuit.tour.ts:226-238`; `en.ftl:1249`, `:1254`
- F70. ERC marks problems on the sheet, red error / amber warning (`ui/tours/schematic/erc.tour.ts:52-55`). **Stage 3a:** "lists them in Problems" does not hold on the default shell: its Problems tab mounts the native body, which lists only core diagnostics (`docs/fix-queue.md:87`, FQ-64; `crates/cadatomic-app/src/problems/native.rs:1-5`); the clean row "Electrical rules check: no violations" (`en.ftl:1256`) is not shown there.
- F71. All passive pins: no driver complaint; ERC does not see a pin with nothing on it — look at the sheet. `ui/tours/concepts.ts:886-889`

## Board

- F72. View ▸ Open as tab ▸ Board brings the board forward. `ui/tours/board/steps.ts:6-21`; `en.ftl:391`, `:623`
- F73. "Pending changes pane" lists what the board hasn't taken; "Acknowledge all"; footprints arrive off the board. `make-led-board.tour.ts(board):71-96`; `en.ftl:874`, `:1008-1009`
- F74. A new design's board is two-layer, 1.6 mm core. `ui/tours/board/make-led-board.tour.ts:101-112`; `en.ftl:1144`
- F75. Fit: Primary+PageDown ("Fit board"). `inventory.toml:2086`; `en.ftl:764`
- F76. Outline: D then S; R rounds corners (1, 2, 3, 5 mm); two clicks. `inventory.toml:2122`; `en.ftl:1151-1152`, `:1240`; `ui/tours/board/shape.tour.ts:43-72`
- F77. Board corners (26, −6) and (44, 6) mm, 18 × 12 mm. `ui/tours/board/make-led-board.tour.ts:12-19`, `:136-143`
- F78. Board HUD shows "X … mm", "Y … mm". `en.ftl:654-655`
- F79. Unplaced chip "{n} footprints off the board · Place them" at the board's top right; brings them beside the outline. `en.ftl:1242-1245`; `crates/cadatomic-app/src/board/pane.rs:922`; `ui/tours/board/place-footprints.tour.ts:43-53`
- F80. Drag by pad 1: R1 to (28.025, 0), J1 to (33.73, 0), D1 to (39.825, 0); courtyards apart. `ui/tours/board/make-led-board.tour.ts:14-22`, `:154-181`
- F81. Routing: P then T, on Top Layer; click pad to start, pad of same net to finish; clicks make corners; Backspace takes back the last part; Esc leaves. `inventory.toml:2113`; `make-led-board.tour.ts(board):191-266`; `ui/tours/board/route-track.tour.ts:45-105`
- F82. Ratsnest: thin straight lines for unrouted connections. `ui/tours/board/route-track.tour.ts:12-15`
- F83. One connection is one undo step. `en.ftl:820`; `route-track.tour.ts:110`
- F84. DRC runs by itself after every change; violations are circles with a cross on the board. `ui/tours/board/drc-markers.tour.ts:9-12`, `:33-35`; `ui/tours/board/live-drc.tour.ts:10-12`. **Stage 3a:** "listed in Problems" fails on the default shell (FQ-64, `docs/fix-queue.md:87`: "No problems." under a tab reading "PROBLEMS 4").
- F85. DRC clean: every connection routed, no clearance breach, nothing off the board. `ui/tours/concepts.ts:382-385`

## Export

- F86. File ▸ Export fabrication outputs… opens the "Fabrication outputs" pane; "Folder", "Choose folder…", "Export". `en.ftl:395`, `:1276-1285`
- F87. Empty folder → the project's outputs/{design}/fab. `en.ftl:1283`; `ui/tours/board/make-led-board.tour.ts:292-311`
- F88. The list: File, What it is, Size, SHA-256; "{count} files written to {dir}". `en.ftl:1288-1292`
- F89. Files include copper-l1-top.gbr, outline.gbr, a .drl drill file, netlist.ipc, board.gbrjob, pick-place.csv, bom.csv (≥10 files for this board). `crates/cadatomic-ecad/src/fab.rs:82-87`, `:192-205`; `ui/tours/board/make-led-board.tour.ts:304-305`
- F90. A board without an outline is refused, with the reason, and nothing is written. `ui/tours/board/export-needs-outline.tour.ts:10-14`, `:47-53`
- F91. Save project at the end. `ui/tours/board/make-led-board.tour.ts:313-318`

## Added in stage 3a (reviser)

- F92. The app picks its shell in `main`: with the default `skelter` feature it runs the native shell; `--egui` runs the eframe shell only in a build with the non-default `egui-shell` feature. `crates/cadatomic-app/src/main.rs:13-17`, `:35-45`; `crates/cadatomic-app/Cargo.toml:85`, `:104`
- F93. The eframe shell is no way round for this tutorial: the ECAD tabs (board, sheet, symbol, library tree, footprint) have only native bodies, and `ViewClass::ui`, the egui body, is an empty default that "draws nothing". `crates/cadatomic-app/src/views/mod.rs:182-185`; `crates/cadatomic-app/src/board/pane.rs:160`, `board/sheet.rs:162`, `library/symbol.rs:97`, `library/tree.rs:77`, `footprint/pane.rs:176`
- F94. The tour's `fill` is ordinary input: click the field, Primary+A, then the text, as a person would type. `crates/cadatomic-app/src/tour/runner.rs:1406-1458`
- F95. `ecad.lib.new` requires `name` (no default) and refuses an empty one. `crates/cadatomic-core/src/pcb_lib.rs:69-72`, `:138-147`
- F96. `ecad.design.new` is a View command whose inputs (`path`, `name`, `template`) are all optional; without a name the design is `untitled-<n>` (the first free n), under the open project's `designs/`, and the project lists it. `crates/cadatomic-core/src/pcb_board.rs:41-59`; `crates/cadatomic-app/src/board/new_design.rs:4-8`, `:37-42`, `:66-70`
- F97. A View command whose required inputs are covered runs at once, without a form; only from the palette with no inputs does it open the form. So File ▸ New ▸ New board design makes `untitled-1` with no Name form, while the palette asks for a name. `crates/cadatomic-app/src/surfaces.rs:552-571`, `:605-606`; native-shell test `crates/cadatomic-app/tests/new_design.rs:176-191` (menu click → `designs/untitled-1`, name `untitled-1`)
- F98. The Board properties grid is a native kind (`<property-grid>`), not the task panel's Svelte form fields: typing on a row starts editing it, Enter commits. `ui/panels/ecad/EcadProperties.svelte:19`; `crates/cadatomic-ui/src/skelter/property_grid.rs:12-17`, `:541-551`
- F99. The New library form's fields are the task panel's `quantity-field`s with commit-only binding (Row.svelte), the fields FQ-96 names. `ui/panels/properties/Row.svelte:4-6`, `:53-66`
- F100. View ▸ Open as tab lists every view class except the pinned ones and the library editors, so it holds "Board", "Board properties" and "Pending changes". `crates/cadatomic-app/src/shell/chrome.rs:52-59`, `:286-297`; `en.ftl:391`, `:623`, `:625`, `:866`; `crates/cadatomic-app/src/board/pending.rs:124-125`
- F101. The pin tool's next pin starts at rotation 0 (pointing right); Space turns it a quarter turn counter-clockwise. `crates/cadatomic-app/src/library/tool.rs:5-6`, `:164-169`; rot 180 is "Left" `crates/cadatomic-app/src/library/pins.rs:24`
- F102. The LED's cathode fab line: Line, Fab, clicks at (−0.6, −0.625) and (−0.6, 0.625), Esc twice. `ui/tours/library/make-led-footprints.tour.ts:213-219`
- F103. The header's 1.70 mm land is the tour author's choice above IPC-2221's 1.40 mm minimum; Samtec gives the 1.02 mm hole. `ui/tours/library/make-led-footprints.tour.ts:136-139`
- F104. Hole is editable only when Pad kind is not smd; the tour sets kind first. `crates/cadatomic-app/src/footprint/props.rs:146`; `ui/tours/library/make-led-footprints.tour.ts:261`
- F105. After Acknowledge all the tour brings the board forward again (`showBoard('board-again')`). `ui/tours/board/make-led-board.tour.ts:100`
- F106. The tour zooms out on the sheet with up to 8 Page Down presses, until the LED's place and the return wire's corner are in view. `ui/tours/schematic/make-led-circuit.tour.ts:116-123`
- F107. The palette's search field takes typing on the native shell (tests type into it and run the chosen command). `crates/cadatomic-app/tests/palette_tree.rs:84-108`, `:250-265`
- F108. There is no README or installer in the repository; `cadatomic` (bin of `cadatomic-app`) opens the window. `crates/cadatomic-app/Cargo.toml:9-11`; `crates/cadatomic-app/src/main.rs:13-14`
- F109. The tour `project.make-led-board` fails on the default shell at "the start page shows" (`docs/fix-queue.md:71`, FQ-61) and on the egui build at naming pin 1 (`docs/fix-queue.md:68`, FQ-57), so the draft no longer points readers to it.
- F110. No workaround for the New library Name field is recorded: FQ-96's fix column says "Nothing" (`docs/fix-queue.md:97`), and `ecad.lib.open` needs an existing library folder, typed into the same kind of form (`en.ftl:1642-1645`).

