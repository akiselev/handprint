# cad-howto-erc: fact sheet

Paths: `cadatomic/` = /home/user/cadatomic, `ecadabra/` = /home/user/ecadabra (the Copperfield ECAD kernel).
Line numbers are from the checkouts as read on 2026-10-26.

## Running the check

- **F1** The sheet pane has a button labelled **Check (ERC)**. (The hint string `ecad-erc-check-hint` exists in the catalog but no code reads it, so users never see it.) — `cadatomic/crates/cadatomic-ui/i18n/en.ftl:1254-1255`; set at `cadatomic/crates/cadatomic-app/src/board/sheet.rs:1440-1444`
- **F2** The button sits at the top right of the sheet (absolute position `top: 4px; right: 94px`), beside the **Follow** toggle at `right: 4px`. — `cadatomic/crates/cadatomic-app/src/board/sheet.rs:1112-1126`; label `en.ftl:877`
- **F3** The same check is the palette command "Check the schematic (ERC)", tooltip "Run the electrical rules check on the active schematic; it then rechecks after every change". Command id `ecad.erc.run`. — `en.ftl:1248-1249`; `cadatomic/docs/behavior/inventory.toml:523`
- **F4** The command palette opens with Primary+Shift+P; Primary is Cmd on macOS and Ctrl elsewhere. — `inventory.toml:1861-1862`, `inventory.toml:1744`
- **F5** No keyboard shortcut is bound to `ecad.erc.run` (only a "command" trigger is listed). — `inventory.toml:4010-4013` (no `key_bindings.item` targets it)
- **F6** `ecad.erc.run` checks the active schematic in a background job and turns the check on for that design; from then on every rebuild of the sheets (an edit, undo or redo) reruns it, newest result wins. — `cadatomic/crates/cadatomic-app/src/board/erc.rs:3-8, 117-137`
- **F7** When a run you asked for finishes, the bottom area opens on the Problems tab, focused at the Schematic section header. — `cadatomic/crates/cadatomic-app/src/app.rs:3225-3234`
- **F8** The status line then reads "Electrical rules check: N errors, M warnings". — `en.ftl:745-751`; `app.rs:3221-3224`
- **F9** The error and warning counts leave out waived violations. — `cadatomic/crates/cadatomic-ecad/src/erc.rs:105-122`
- **F10** The app runs Copperfield's standard check set `Erc::basic()`. — `cadatomic/crates/cadatomic-ecad/src/erc.rs:140`
- **F11** `Erc::basic()` = connection matrix, multiple names, single-pin nets, no driving source, floating input pins, floating net labels, duplicate designators, drawing-vs-circuit mismatch, multiply placed sheets. — `ecadabra/crates/schematic/src/erc.rs:144-158`
- **F12** Checks and default levels follow Altium Designer's (Error Reporting and Connection Matrix). — `cadatomic/ui/tours/schematic/erc.tour.ts:19-20`; `ecadabra/crates/schematic/src/erc.rs:9-14`
- **F13** A clean run lists one info row "Electrical rules check: no violations". — `en.ftl:1256`; `cadatomic/crates/cadatomic-app/src/problems/rows.rs:82-99`
- **F14** If the schematic cannot be flattened, the row is "The electrical rules check could not read the schematic" and there are no markers. — `en.ftl:1257`; `rows.rs:62-80`; `cadatomic-ecad/src/erc.rs:140-148`
- **F15** The bottom area is also reachable from View ▸ "Problems and jobs" and the status-bar toggle "Show problems". — `en.ftl:390`, `en.ftl:364`

## Reading the results

- **F16** Each violation is a row in Problems' **Schematic** section (`problems-section-schematic = Schematic`). — `en.ftl:567`; `rows.rs:132-146`
- **F17** The row shows the severity word, the check's catalog name (e.g. "Nets with only one pin") and a location "<schematic> · <sheet> · (x, y) mm". — `cadatomic/ui/panels/problems/ProblemsPanel.svelte:132-137`; `rows.rs:121-130`
- **F18** The kernel's own sentence with net and pin names (e.g. "Net N0 has only one pin (R1-1)") and any waiver line are stored as the row's `detail`; the Problems panel markup does not render `detail`. — `rows.rs:116-120, 536-539`; `ProblemsPanel.svelte:113-139` (no use of `detail`)
- **F19** Errors come first, then warnings. — `cadatomic-ecad/src/erc.rs:99, 183-184`
- **F20** On the sheet each place is marked with a ring and a cross: red for an error, amber for a warning, drawn thicker for the revealed one. — `cadatomic-app/src/board/erc.rs:286-316`
- **F21** Clicking a row runs `ecad.erc.reveal`: shows the violation's sheet, frames its places (at least 10 mm around each) and selects the items there. — `erc.tour.ts:66-77`; `cadatomic-app/src/board/erc.rs:218-264`; `cadatomic-ecad/src/erc.rs:29-31`
- **F22** Enter or a double-click opens a row; the Menu key, Shift+F10 or a secondary click opens its menu. — `ProblemsPanel.svelte:4-6`; `erc-waive.tour.ts:54-56`
- **F23** The row menu holds **Open**, then **Waive…** (waivable row) or **Remove waiver** (waived row), then fixes or a disabled **No fix available**. ERC rows carry no fixes. — `ProblemsPanel.svelte:142-155`; `en.ftl:561-562, 1071-1072`; `rows.rs:144` (`fixes: Vec::new()`)
- **F24** Violations that name no drawn place (e.g. multiple names) have no sheet marker and are listed only. — `cadatomic-ecad/src/erc.rs:11-13`

## The checks (catalog names, default levels, what triggers them)

Catalog names: `en.ftl:1258-1268`. Default levels: `ecadabra/crates/schematic/src/erc/catalog.rs` rows.

- **F25** "Nets with only one pin" (`erc.single_pin_net`), default **Error**: a net with exactly one pin plus a wire or a naming object. Message "Net {net} has only one pin ({pin})". — `catalog.rs:876-886`; `checks.rs:312-339`
- **F26** A pin with no wire at all is not a single-pin net (the check needs a wire or another item). — `checks.rs:326`
- **F27** "Unconnected objects in net" (`erc.unconnected_net_item`), level taken from the connection matrix (catalog default Warning, `uses_matrix: true`). Message "Unconnected {code} {item}". — `catalog.rs:276-286`; `checks.rs:172-205`
- **F28** In the default matrix, the "Unconnected" column reports Input pins, Input ports and sheet entries as Warning, and Passive, IO, Output, Power pins as no-report. — `ecadabra/crates/schematic/src/erc/config.rs:122-140` (row `L17` = "WNNNNNNNWNNNWWWWN"), `config.rs:176-178`; code order `ecadabra/crates/model/src/erc.rs:108-126`
- **F29** The default matrix is Altium's as 9 of 11 example projects store it; its grade is "inferred". — `config.rs:119-121`
- **F30** "Nets with no driving source" (`erc.net_with_no_driving_source`), default **Warning**. Drivers: IO, Output, Open Collector, HiZ, Open Emitter, Power pins; Input/Bidirectional ports; Output/Bidirectional sheet entries. Net labels and power ports count as passive. Reported when the net has items, no driver, and not every item is passive. — `catalog.rs:288-298`; `checks.rs:341-395` (line 372: Label/Power → passive)
- **F31** "Nets containing floating input pins" (`erc.floating_input_pins_on_net`), default **Error**: every pin on the net is an input and there is no power port on it. — `catalog.rs:300-310`; `checks.rs:397-434`
- **F32** "Nets containing multiple similar objects" (`erc.multiple_same_connection_code_on_net`), matrix level; default Error. Message "Net {net} contains {n} {code} objects (...)". Default matrix: Output–Output = Error; Power–Power and Input–Input = no report. — `catalog.rs:324-334`; `checks.rs:149-169`; `config.rs:123,125,130` (rows L1[0]='N', L3[2]='E', L8[7]='N')
- **F33** "Nets with possible connection problems" (`erc.different_connection_codes_on_net`), matrix level; default Error. Message "Net {net} contains {code} and {code} objects (...)". Default matrix examples: Output+Power pin = Error, Output+Open Collector = Error, Output+IO = Warning, Input+Output = no report. — `catalog.rs:312-322`; `checks.rs:84-107`; `config.rs:124-130` (L3[0]='N', L3[1]='W', L4[2]='E', L8[2]='E')
- **F34** Connection-code display names: "Input Pin", "IO Pin", "Output Pin", "Open Collector Pin", "Passive Pin", "HiZ Pin", "Open Emitter Pin", "Power Pin", ports, sheet entries, "Unconnected". — `ecadabra/crates/model/src/erc.rs:199-218`
- **F35** A pin's code comes from its electrical type; unspecified counts as Passive; both power types count as Power. — `ecadabra/crates/model/src/erc.rs:161-175`
- **F36** Pin electrical type is edited in the library symbol editor ("Electrical type"). — `en.ftl:1677, 1683, 1777`
- **F37** "Nets with multiple names" (`erc.multiple_names_for_net`), default **Warning**: a net carries more than one name from net labels, power ports, (optionally) ports and sheet entries, or names typed on nets; compared case-insensitively. Message "Nets with multiple names: A, B". No drawn place. — `catalog.rs:336-346`; `checks.rs:245-310`
- **F38** "Floating net labels" (`erc.floating_net_label`), default **Warning**: a label whose drawn group holds no wire, pin, port or power port. Message "Floating net label {text}". — `catalog.rs:852-862`; `checks.rs:436-486`
- **F39** "Duplicate part designators" (`erc.duplicate_component_designator`), default **Error**: two placed parts with one designator, case-insensitive; two `R?` count too. No drawn place. — `catalog.rs:552-562`; `checks.rs:488-532`
- **F40** "Drawing disagrees with the circuit" (`erc.connectivity_mismatch`), default **Error**, Copperfield's own: a connected group of drawn items on two nets ("Drawn connection joins N nets"), or "A pin touches a net it is not connected to". No drawn place. — `config.rs:84-96`; `checks.rs:534-577`
- **F41** "Sheet placed several times in Flat or Global scope", default Error. — `config.rs:97-108`; `en.ftl:1268`
- **F42** Error and Fatal levels show as errors; Warning as warnings. — `cadatomic-ecad/src/erc.rs:172-175`
- **F43** The app has no settings UI for ERC levels or the matrix; a design can store its own levels (`erc_level`, `erc_matrix` lines) which then apply. — no such strings in `en.ftl` (only the comment at 1247); `ecadabra/crates/schematic/src/erc/config.rs:4-7, 246-272`

## Fixing tools

- **F44** Wire tool P then W; net label P then N; power port P then O; no-connect P then V; junction P then J; annotate T then A. — `inventory.toml:2689, 2698, 2707, 2743, 2752, 2644`
- **F45** Wire tool hint: "Click adds a corner · click twice or Enter ends the wire · Space flips the corner · Shift+Space: 90° or any angle · Backspace removes a corner · Esc ends". — `en.ftl:1034`
- **F46** After a fix, the rerun clears the fixed violations by itself (tour: wiring R1-2 to R2-1 drops two errors). — `erc.tour.ts:111-122`
- **F47** Pins under a no-connect marker, and pins of type `nc`, are left out of ERC. — `ecadabra/crates/schematic/src/erc/nets.rs:7-8, 246-250`
- **F48** No-connect hint: "Click a pin's end to mark it as meant to stay unconnected · Esc ends"; command "Place no-connect marker". — `en.ftl:1031, 981-982`
- **F49** A power port connects what it touches to a supply net; every power port with one name is one net on every sheet; starts as GND. — `power-port.tour.ts:8-14, 30-34, 44-49`
- **F50** Crossing wires are not connected; a junction joins them. Junction hint: "Click a wire to place a junction: wires crossing there connect · Esc ends". — `junction.tour.ts:8-14`; `en.ftl:1032`
- **F51** Two net labels with the same name connect their nets without a wire. — `net-label.tour.ts:11-14`
- **F52** Annotate: "Give every part still called R?, C? or U? its designator; locked designators stay". — `en.ftl:960-961`
- **F53** A designator can be changed in the Properties pane (palette "Properties pane"; double-click Designator, select the old text with Primary+A, type, Enter). The pane opens as a tab over the sheet; click the sheet's tab to see the sheet again. — `cadatomic/ui/tours/schematic/properties.tour.ts:49-57, 77-85`; `en.ftl:875`
- **F54** Primary+click (Ctrl, Cmd on macOS) a pin lights its whole net on the sheet and on the design's board. — `cadatomic/ui/tours/schematic/cross-probe.tour.ts:51-60` (modifier `primary` at :57)

## Waivers

- **F55** "Waive…" turns the row into an inline editor with hint "Why is it acceptable?", buttons **Waive** and **Cancel**; Enter or Waive records, Escape or Cancel drops. — `en.ftl:1071-1075`; `ProblemsPanel.svelte:9-11, 73-86, 119-130`
- **F56** An empty (or blank) reason records nothing; the editor just closes. — `ProblemsPanel.svelte:77-80`; `cadatomic-app/src/board/erc.rs:67-69`
- **F57** A waiver is a No ERC directive at the violation's first place, for that violation's kind only, with reason, author and today's date (YYYY-MM-DD). — `cadatomic-ecad/src/erc.rs:205-249`; `cadatomic-app/src/board/erc.rs:12-17`
- **F58** The author is the OS login name (`USER`, `USERNAME`), else "unknown". — `cadatomic-app/src/board/waivers.rs:12-19`
- **F59** A waived violation stays in Problems with severity text **Waived** (info); it is not counted among errors/warnings. — `rows.rs:102-106`; `en.ftl:1954`; `cadatomic-ecad/src/erc.rs:16-20, 105-122`
- **F60** A waived violation's marker leaves the sheet. — `cadatomic-app/src/board/erc.rs:272-277`
- **F61** Waiver line text "Waived by {author} on {date}: {reason}" is in the row's detail (not rendered by the panel, see F18). — `en.ftl:1065`; `rows.rs:118-119, 177-183`
- **F62** Only a violation drawn somewhere can be waived; rows with no place get no Waive… item. — `rows.rs:115-116`; `cadatomic-ecad/src/erc.rs:226-239`
- **F63** A waiver matches every violation of its kind that names an item at its point (so a later violation of that kind at that point is covered too). — `ecadabra/crates/schematic/src/erc/nets.rs:147-158`; `ecadabra/crates/schematic/src/erc.rs:15-19, 221-222`
- **F64** **Remove waiver** (`ecad.erc.unwaive`, palette "Remove ERC waiver", tooltip "Delete a waiver's No ERC directive: its violation counts again"). — `en.ftl:1072, 1950-1951`; `cadatomic-app/src/board/erc.rs:99-110`
- **F65** Waive and remove are edits; Undo (Primary+Z) takes them back. — `cadatomic-app/src/board/erc.rs:16-17`; `cadatomic-ecad/src/erc.rs:20`; `cadatomic/ui/tours/schematic/erc-waive.tour.ts:109-119`; `inventory.toml:1888-1889`
- **F66** The palette command "Waive ERC violation" exists (synonyms "no erc, waive, accept violation, ignore erc"), taking design, sheet, location, kinds, reason, author, date. — `en.ftl:1939-1949`
- **F67** The No ERC directive is not drawn on the sheet by the app's schematic scene (no directive handling in `cadatomic-ecad/src/sch.rs`). — grep of `cadatomic-ecad/src/sch*.rs` for `directive|noerc`: no hits

## Tours

- **F68** Help ▸ "Tours…" lists the guided tours; the ERC ones are "Check the circuit (ERC)" and "Waive an ERC violation, with a reason". — `en.ftl:1087`; `erc.tour.ts:29`; `erc-waive.tour.ts:27`

## Command line (kernel)

- **F69** The kernel CLI `copperfield erc [--summary] <design-dir | file.PrjPcb | file.SchDoc>...` runs the same `Erc::basic()`; prints counts per kind and each violation with its level, marking waived ones "(waived)"; exits 1 when an unwaived violation is an error or fatal. — `ecadabra/crates/cli/src/main.rs:17, 53-56, 907-933`

## Added by the reviser (pass 1)

- **F70** "Drawing disagrees with the circuit" violations carry a sheet but no place (`cx.violation` without `.at()`), so the row's location is only the schematic's name, there is no marker, and a reveal leaves the view where it is (it acts only when the violation has a first spot). — `ecadabra/crates/schematic/src/erc/checks.rs:560-574`; `cadatomic/crates/cadatomic-app/src/problems/rows.rs:125-131`; `cadatomic/crates/cadatomic-ecad/src/erc.rs:68-70`; `cadatomic/crates/cadatomic-app/src/board/erc.rs:248`
- **F71** A design keeps pinned copies of its parts; "Update from library" replaces them with the libraries' current versions. In the app, the **Components library** pane (palette "Components library") lists "This design" parts with a status such as "Out of date" and an **Update** button (and "Update all out of date"). — `en.ftl:1396-1397, 1404-1406, 1409, 1419, 1433-1434`; `cadatomic/crates/cadatomic-app/src/board/lib_pane.rs:4-14`
- **F72** The schematic tool keys (P W, P N, P O, P V, P J, T A) are bound on the **Viewport** layer, and Page Down is read by the sheet canvas; the tours point at or click the sheet first. — `cadatomic/docs/behavior/inventory.toml:2643-2760`; `cadatomic/crates/cadatomic-app/src/board/sheet.rs:439-442`; `cadatomic/ui/tours/schematic/erc.tour.ts:91-108`
- **F73** The check stays on only for the open schematic: the "on" flag is session state, and closing the schematic's pane drops it. — `cadatomic/crates/cadatomic-app/src/board/erc.rs:33-35, 124-128`; `cadatomic/crates/cadatomic-app/src/board/hub.rs:644-655`
- **F74** The app's check uses the levels and matrix the design stores (`Erc::run` reads `ErcOptions::from_store`); an Altium import writes them from the PrjPcb. — `ecadabra/crates/schematic/src/erc.rs:179-185`; `ecadabra/crates/schematic/src/erc/config.rs:4-7`; `cadatomic/crates/cadatomic-ecad/src/erc.rs:140`
- **F75** "Unconnected objects in net" catalog default is Warning (matrix column L17). — `ecadabra/crates/schematic/src/erc/catalog.rs:276-286`; `config.rs:139`
- **F76** A single-pin net is any net with one pin and a wire or a non-pin item, so one pin plus a power port whose name no other pin uses is reported. — `ecadabra/crates/schematic/src/erc/checks.rs:323-326`
- **F77** "Nets with no driving source": a net whose items are all passive (passive pins, labels, power ports) is not reported. — `ecadabra/crates/schematic/src/erc/checks.rs:367-378`
- **F78** The `copperfield` CLI is not part of cadatomic (no `cf-cli` dependency or install script). It prints each violation's full message unless `--summary`, and the input may be a design folder, PrjPcb or SchDoc. — `ecadabra/crates/cli/src/main.rs:17, 53-56, 907-933`; grep of `cadatomic/Cargo.toml`, `cadatomic/crates/*/Cargo.toml`, `cadatomic/scripts` for `cf-cli`: no hits
- **F79** Undo is Primary+Z on the Global layer. — `cadatomic/docs/behavior/inventory.toml:1887-1890`
- **F80** The power port tool's strip field is labelled "Name". — `en.ftl:1035`
