# Make an LED board: from an empty project to fabrication files

You will build a small board by hand: an LED, the 330 Ω resistor that limits its current, and a
two-pin header that brings in 5 V. You draw the symbols and footprints, join them into parts, draw
and check the schematic, then lay out, route, check and export an 18 × 12 mm board. Allow about an
hour.

The same walkthrough is a guided tour in the app: **Help ▸ Tours…**, then **Make an LED board,
from nothing**, with **Watch it** or **Try it**.

## Before you start

- **Two-key shortcuts.** "P then W" means press P, release it, then press W. The pointer must be
  over the canvas you are working on.
- **Ctrl** below means Cmd on macOS.
- **The command palette** finds any command by name. Click **Search commands…** in the title bar
  (Ctrl+Shift+P also opens it).
- **Zooming.** Page Down zooms out at the pointer, Page Up zooms in.
- **Undo** is Ctrl+Z.

The app is changing quickly, and two things are known to misbehave in current builds. Naming a
pin with Tab while you place it sometimes doesn't take; step 2 shows a way round that. Typing
into the **Name** field of some forms, such as **New library**, can also fail to register, and
there is no way round that one yet.

## 1. Start a project

A project is one folder for everything about one product.

1. With nothing open, the start page shows **New project…**, **Open project…** and **New part**.
   Click **New project…** (or use **File ▸ New ▸ Project…**).
2. In the folder dialog, make a new folder called `led` and choose it.

The project opens, named `led` after its folder, with no designs or libraries yet. The folder now
holds `project.toml`, which lists what the project contains.

## 2. Draw the three symbols

A symbol is a part's drawing on the schematic: a body and pins.

### Make the symbol library

1. Choose **File ▸ New ▸ New library**.
2. In the form that appears, type `led-symbols` in **Name** and click **Accept**.

The library goes into the project's `libraries/` folder and its tree opens.

### Create a symbol and place its pins

Start with the LED. In the library tree, type `LED` in **New symbol** and `D` in **Prefix**, then
click **Create**. The prefix names placed copies: this LED will become D1. The empty symbol opens
with a small cross at its origin; pins snap to a 100 mil grid.

1. With the pointer on the symbol, press **P then P** for the pin tool. The first pin is number 1.
2. Press **Space** twice so the pin points left. Each press turns it a quarter turn.
3. Press **Tab**. Properties shows **Next pin**. Click its **Name** row and type `A`, then press
   Enter.
4. Click two grid squares left of the origin (−200, 0 mil). Pin 1 is placed.
5. Press **Space** twice so the next pin points right, press **Tab**, name it `K`, and click two
   squares right of the origin (200, 0).
6. Press **Esc** to end the pin tool.

New pins are **passive**, which is right for all three parts: none drives a net or needs a driver.

If a placed pin's name is still its number, the Tab step did not take. Click **Pin table** above
the symbol, type the name into that pin's **Name** cell and press Enter.

### Set the value and draw the body

With nothing selected, Properties shows the symbol itself; set its **Value** to `LED`. Then draw
the body; Esc ends each tool:

- **P then Y** draws a polygon: click each corner, then Enter.
- **P then L** draws a line: click both ends.
- **P then R** draws a rectangle: click two opposite corners.
- **P then T** places a text, `${REF}` by default (each copy shows its designator there). For a
  value text, press Tab, set **Text** to `${VALUE}`, then click.

### The three symbols

Coordinates are in mil from the origin cross; negative y is below it.

| Symbol | Prefix | Value | Pins | Body |
|---|---|---|---|---|
| `LED` | D | LED | 1 `A` at (−200, 0) pointing left; 2 `K` at (200, 0) pointing right | triangle (−100, 100), (−100, −100), (100, 0); line (100, 100)–(100, −100); `${REF}` at (−100, 150); `${VALUE}` at (−100, −200) |
| `R` | R | R | 1 at (−200, 0) left; 2 at (200, 0) right | rectangle (−100, −50)–(100, 50); `${REF}` at (−100, 100); `${VALUE}` at (−100, −150) |
| `Conn_01x02` | J | Conn_01x02 | 1 at (−200, 0) left; 2 at (−200, −100) left | rectangle (−100, 50)–(0, −150); `${REF}` at (−100, 100); `${VALUE}` at (−100, −250) |

The resistor's and connector's pins need no names: a new pin's name is its number.

Press **Ctrl+S** to save the library, then **File ▸ Save project** so the project lists it.

## 3. Draw the three footprints

A footprint is the part's land pattern on the board. The sizes come from Kingbright's APT2012EC
(LED), Vishay's CRCW (resistor) and Samtec's TSW (header) datasheets.

1. Make a second library: run **New library** again (from **File ▸ New** or the palette), name it
   `led-footprints` and click **Accept**.
2. In the library tree, type the footprint's name in **New footprint** and click **Create
   footprint**. It opens empty; the canvas shows the pointer's X and Y in mm.

For each footprint, in this order:

1. **Pads.** Click **Pad**, then press **Tab**: Properties shows the next pad. Set **Width (mm)**
   and **Height (mm)** (and **Pad kind**, **Hole** or **Shape** where the table says), then click
   where each pad goes. Pads number themselves. Esc ends the tool.
2. Click **Origin: centre**: the reference point moves to the pads' centre, where pick-and-place
   expects it.
3. **Fab outline:** click **Line**, then **Fab**, then the body's four corners and the first again.
   Esc twice ends the line, then the tool.
4. **Designator:** click **Text**, then **Silkscreen**, then above the part. It reads REF until
   the part is on a board.
5. **Courtyard:** click **Courtyard from pads…** (0.25 mm round pads and body, IPC-7351's nominal
   margin), then **Accept courtyard**.

| Footprint | Pads (mm) | Pad centres (mm) | Fab box (mm) | Designator |
|---|---|---|---|---|
| `LED_0805` | 1.25 × 1.1 | (−1.175, 0), (1.175, 0) | corners ±1, ±0.625, plus a fab line across the body at x = −0.6 to mark the cathode | (0, 1.6) |
| `R_0805` | 0.95 × 1.45 | (−0.975, 0), (0.975, 0) | corners ±1, ±0.625 | (0, 1.6) |
| `PinHeader_1x02_P2.54mm` | Pad kind `th`, 1.7 × 1.7, Hole 1.02 | (0, 0), then (2.54, 0) | corners ±2.54, ±1.27 (after centring) | (0, 2.4) |

On the LED, pad 1 (left) is the cathode, as in the datasheet.

The header needs extra moves. Before its pads, click **Grid** four times, to 1.27 mm. Place pad 1,
square, at the origin; then set **Shape** to `circle` and click 2.54 mm right for pad 2, so pin 1
stands out. After the fab outline, click **Grid** once (back to 0.025 mm) and draw a
**Silkscreen** box just outside the body, corners ±2.625, ±1.325.

Press **Ctrl+S**, then **File ▸ Save project**. The project now lists both libraries.

## 4. Join symbols and footprints into parts

A part is what the schematic places: a symbol, a pinned copy of a footprint, and a pin-to-pad
map. The parts live in `led-symbols`.

1. In the library tree, select the `LED` symbol. A **New part from LED** form appears.
2. Set **Footprint library** to `led-footprints` and **Footprint** to `LED_0805`. Leave **Name** as
   `LED`. Click **Create part** (scroll the tree down if it is out of sight).
3. The part opens in its own tab. Type `LED` as its **Value** and press Tab.
4. Fix the pin map. A new part maps each pin to the pad of the same number; each pin's row has a
   toggle per pad. Pin 1 is the anode but pad 1 is the cathode, so on pin 1's row turn pad 1 off
   and pad 2 on, and on pin 2's row the reverse.

Make the other two the same way:

- `R` from symbol `R` and `R_0805`, value `R`; the map by number is right.
- `J` from symbol `Conn_01x02` and `PinHeader_1x02_P2.54mm`, value `Conn_01x02`.

Each part's footprint status reads **Up to date** (it would read **Out of date** after a change in
`led-footprints`). Press **Ctrl+S**.

## 5. Draw the circuit

1. Choose **File ▸ New ▸ New board design** (or run it from the palette). Name it `led` and click
   **Accept**. An empty schematic sheet opens.
2. With the pointer on the sheet, press **P then P**. In the strip at the bottom, open the
   **Part** list and choose `J` from `led-symbols`. Click to place it, then press Esc.
3. Press Page Down a few times to make room on the left.
4. Press **P then P**, choose `R`, and click six grid squares left of the connector. Esc.
5. Press **P then P** and choose `LED`. Press **Space** twice to turn it half round, so its anode
   faces the resistor, then click six squares left of the resistor. Esc.
6. Press **T then A** to annotate: `J?`, `R?` and `D?` become J1, R1 and D1.

Now wire one loop, out of J1's pin 1, through resistor and LED, back into pin 2. Each wire joins
the pins it touches into one **net**.

1. Press **P then W** to start the wire tool.
2. Click the end of J1's pin 1, then the end of R1's pin 2, and press Enter.
3. Click R1's pin 1, then D1's pin 1 (the anode), and press Enter.
4. Click D1's pin 2 (the cathode), click on the empty sheet below D1 and again below J1 to make
   two corners, click J1's pin 2, and press Enter. Press Esc to end the tool.

That makes three nets in one loop.

Last, the resistor's value. From 5 V the LED takes about 2 V, leaving 3 V across 330 Ω: about
9 mA. Click R1, run **Properties pane** from the palette, double-click **Value**, type `330 Ω` and
press Enter.

## 6. Check the schematic

Click **Check (ERC)** at the top right of the sheet, or run **Check the schematic (ERC)** from the
palette. Anything it finds is marked on the sheet (red for errors, amber for warnings) and
listed in Problems.

It should find nothing: every pin is passive and every net joins two pins. It cannot see a pin
with nothing on it at all, so look over the sheet too.

Press **Ctrl+S** with the pointer on the sheet, then run **Save project**.

## 7. Bring the circuit onto the board

The board never changes behind your back: schematic changes wait until you accept them.

1. Choose **View ▸ Open as tab ▸ Board** to bring the board to the front.
2. Run **Pending changes pane** from the palette. It lists the three parts and the three nets.
3. Click **Acknowledge all**. The board takes the footprints and their nets; they lie off the
   board for now.

The board has two copper layers on a 1.6 mm core; this one uses only the top.

## 8. Shape the board and place the parts

1. Press **Ctrl+Page Down** to fit the view on the three footprints.
2. With the pointer at the parts' left end, press Page Down three times to make room on the right.
3. Press **D then S** to start the outline tool, and press **R** once for 1 mm rounded corners.
   Each further press steps the radius: 2, 3, 5 mm, then square again.
4. Click the corner at (26, −6) mm and the opposite corner at (44, 6) mm, using the X and Y readout.
   The board is 18 × 12 mm.
5. At the board's top right, click **3 footprints off the board · Place them**. The footprints
   move into a column beside the outline.
6. Drag each part by its pad 1 onto the board's middle line: R1 to (28.025, 0), J1 to (33.73, 0)
   and D1 to (39.825, 0).

The line round each footprint is its **courtyard**, the room it needs; keep courtyards apart. In
this order no connection crosses another, so every track fits on the top layer.

## 9. Route the tracks

The thin straight lines are the **ratsnest**: connections not yet made in copper.

1. Press **P then T** to route. Tracks go on the Top Layer at the default width, plenty for a
   few milliamperes.
2. Click R1's pad 2, then J1's pad 1. The first connection is copper.
3. Click J1's pad 2, then D1's pad 1, the LED's cathode.
4. Click D1's pad 2. Click above D1 at (42.175, 4), then above R1 at (28.025, 4): each click fixes
   a corner. Click R1's pad 1 to finish.
5. Press Esc to leave the tool.

No ratsnest line is left. If a corner goes wrong, Backspace takes back the last piece; a finished
connection is one undo step.

## 10. Check the board and export

The design rule check runs by itself after every change. A violation shows as a circle with a
cross and a row in Problems. Yours should show none: the board is **DRC clean**, every connection
routed, no copper too close to another net, nothing outside the board.

1. Choose **File ▸ Export fabrication outputs…**. The **Fabrication outputs** pane opens.
2. Leave **Folder** empty and click **Export**.

The files go into the project's `outputs/led/fab`, and the pane lists each with what it is, its
size and its SHA-256: a Gerber file per layer (`copper-l1-top.gbr`, `outline.gbr` and so on), a
drill file, the bare-board test netlist `netlist.ipc`, the job file `board.gbrjob`, and
`pick-place.csv` and `bom.csv` for assembly.

If Export refuses, its message says why (a board without an outline, for one), and nothing is
written.

Finish with **File ▸ Save project**. Your board is ready to send.
