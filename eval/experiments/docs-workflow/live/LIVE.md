# Live verification: cadatomic under Xvfb (2026-10-26)

Shots: `/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/exp/live/gq/shots/`
(prefix `SH` below). gq action log: `.../exp/live/gq/actions.ndjson`.

## What ran

| Repo | Checkout in /home/user | Pin (pins.toml) | Built from |
|---|---|---|---|
| cadatomic | b350c24 | (n/a) | b350c24 (`git clone --shared`, scratch `ws/cadatomic`) |
| cadabra3 | 7bf8908 (**not the pin**) | b02142a | b02142a (shared clone in scratch `ws/`) |
| svelters | 6c1ea458 (**not the pin**) | 6689112c | 6689112c (shared clone, `pnpm install --frozen-lockfile`) |
| resolvent | **missing** | b604f93 | b604f93 (cloned from GitHub into scratch `ws/`) |
| parasolid-rs | **missing** | 02c5699 | 02c5699 (cloned from GitHub into scratch `ws/`) |
| ecadabra (Copperfield) | 39187fc | 2b3c58c (git dep) | cargo fetched rev 2b3c58c |
| berth / quire / altium-cli | (git deps) | 34e4e30 / ed0665b / 4d2450d (lock) | cargo fetched |

The /home/user checkouts weren't touched. The build ran from a scratch workspace
(`.../scratchpad/ws/`) whose siblings are at the pinned shas. Container changes:
`git config --global url."https://github.com/".insteadOf "ssh://git@github.com/"` (as CLOUD.md §3
describes), `add_repo akiselev/berth`, and apt packages for gq (at-spi2-core, python3-pyatspi,
gir1.2-atspi-2.0, dbus-x11, x11-utils, openbox, xclip, python3-numpy, python3-pil, zenity).

**Build:** about 4 min 49 s (289 s wall). This was a debug build with `CARGO_PROFILE_DEV_DEBUG=0` and
no sccache, using `cad-target`, which already held some build output from earlier in the session.

```sh
cd .../scratchpad/ws/cadatomic
CARGO_TARGET_DIR=.../scratchpad/cad-target CARGO_PROFILE_DEV_DEBUG=0 CARGO_NET_GIT_FETCH_WITH_CLI=true \
  scripts/cargo.sh build -p cadatomic-app --bin cadatomic -j4 --locked
GQ_DIR=.../exp/live/gq WGPU_BACKEND=vulkan tools/gui-qc/gq up --size 1600x1000 --wm -- .../cad-target/debug/cadatomic
# then gq click / key / type / shot / find / locate / text (real XTEST input; no RPC, MCP or tours used to act)
```

System lavapipe was used (Mesa from apt), with a fresh XDG profile from gq and openbox as the
window manager. Each frame took about 1–2 s to redraw after input, so every screenshot was taken
after a wait.

## Q1: typing into the New library Name field: WORKS

- Start page ▸ **New project…**. The OS folder dialog opened, because rfd fell back to zenity once
  it was installed. I typed the folder path and the project opened (`SH/0005-07-project-open.png`).
  GTK's location-bar autocomplete garbled the path, so the project is called `ome-user` inside
  `led/`. That came from my input and isn't an app bug.
- **File ▸ New ▸ New library** opened the form (Name, Folder, Accept, Cancel) in the right-hand
  Properties pane (`SH/0008-10-new-library-form.png`).
- Clicked **Name** and typed `led-symbols`. The text showed in the field
  (`SH/0009-11-name-typed.png`). Pressed Tab, then clicked **Accept**: status read "Library
  opened", a **Library** tab opened with `led-symbols` (Symbols 0 / Footprints 0 / Parts 0), and
  `libraries/led-symbols/library.ecad` appeared on disk (`SH/0011-13-after-accept.png`).
- Repeated with `led-footprints`, pressing Enter without Tab. The library was created
  (`SH/0012-14-second-library-enter.png`).
- Also typed into the library tree's **New symbol** field (`LED`) and **Prefix** field (`D`), then
  clicked **Create**. The symbol `LED (D)` opened (`SH/0013-15-new-symbol.png`).
- A possible lead on FQ-96: while the field showed `led-symbols`, gq's renderer probe (`gq text`)
  read `field-name` as `value: ""` and the status line still said "New library: fill in name".
  AT-SPI meanwhile reported `entry "Name" ='led-symbols'`, and Accept used the typed name. The
  tour runner's readback of the field's value may be what's broken, not the typing itself. This is
  an inference and wasn't checked further.

## Q2: Ctrl+Shift+P: WORKS

- Pressed Ctrl+Shift+P on the start page. The **Command palette** opened with its search field
  focused (`SH/0001-02-ctrl-shift-p.png`). The title-bar search box also shows the hint
  "Ctrl+Shift+P".
- Typing into the palette worked, though the list took about 2 s to redraw
  (`SH/0002-03-palette-typed.png` mid-redraw, `SH/0003-04-palette-after-wait.png` filtered).

## Q3: Problems tab and ERC: rows WORK; row menu / Waive… BROKEN

- **File ▸ New ▸ New board design** created `untitled-1` and opened its schematic with **Check
  (ERC)** and **Follow** at the top right (`SH/0018-20-new-design.png`).
- Placed a net label with P then N. With the pointer only hovering over the sheet, the status
  stuck at "P was pressed; waiting for the next key" and N did nothing (`SH/0021-23-...`). After
  one click on the sheet to focus it, P then N started the Net label tool
  (`SH/0022-24-netlabel-tool-c.png`). Clicked to place `NetLabel1`, then Esc.
- Clicked **Check (ERC)**. The bottom area opened on **PROBLEMS** with a "Show waived" checkbox, a
  "Schematic  1 problem" header and the row **"Warning  Floating net labels  untitled-1 · Sheet1 ·
  (0.0000, 5.0800) mm"**. The status line read "Electrical rules check: 0 errors, 1 warning", and
  an amber ring-and-cross marker sat on the label (`SH/0023-25-after-erc.png`). The right-hand
  status counter still read "0 errors, 0 warnings".
- **Row selection by mouse is wrong:** left- or right-clicking the warning row, or the empty area
  below it, always highlights the **header** row "Schematic 1 problem" (`SH/0024-26-row-menu.png`,
  `SH/0025-27-row-leftclick.png`, `SH/0030…0032`). A hit-test at the row (`gq hit 300,802`)
  reaches the warning's listitem.
- The **keyboard** works: Down selects the warning row (`SH/0026-28-down.png`), and selecting it
  revealed the label on the sheet ("1 object selected", view zoomed; `SH/0028-30-menu-key.png`).
- **No row menu ever opened:** right-click, Shift+F10 and the Menu key were each tried with the
  warning row selected. The accessibility tree holds an empty `menu` node of height 0, and no
  "Waive…" item exists anywhere (`gq find 'name~=Waive'` finds nothing). `ProblemsPanel.svelte`
  defines a `<context-menu>` with `waive`/`unwaive` actions, but it never showed live.
- **The ERC waiver reason field:** unreachable, because Waive… never appears. Not tested.
- Side finding: **Help ▸ Tours…** and the palette's "Tour: Check the circuit (ERC)" both did
  nothing but set the status to "No part is open; New part makes one" (`SH/0014-16-tours.png`,
  `SH/0017-19-erc-tour-start.png`). That was with a project, two libraries and a symbol editor
  open, and no MCAD part.

## What this means for the two pages

- LED tutorial: the "Known blocker" claim (Name stays empty, only step 1 can be finished) is wrong
  for a person typing on Linux/X11. The note in step 6 ("the Problems tab may not list the results
  of this check") is also wrong: the rows are listed.
- ERC how-to: Run the check, the Problems rows, the status line and the markers all match. "Click a
  row: the sheet switches…" doesn't hold with the mouse, because a click selects the header; the
  keyboard does reveal the place. The "Waive a finding" and "Remove a waiver" sections can't be
  done live: no row menu opens. The "Known problem" claim blames the reason field, but the real
  blocker comes earlier, at the missing menu. The page's "Tours" pointer led only to "No part is
  open".
