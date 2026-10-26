# Make an LED board: from an empty project to fabrication files

**Known blocker:** in current builds, text typed into the **Name** field of the **New library**
form doesn't reach it (the app's own automated runs fail there every time), and there is no way
round it yet. If that happens to you, you can't make the libraries in steps 2 and 3, which steps 4
to 10 build on, so only step 1 can be finished today.

You will build a small 18 × 12 mm board by hand, from symbols to fabrication files. It carries an
LED, the 330 Ω resistor that limits its current, and a two-pin header that brings in 5 V. Allow
about an hour.

## Before you start

There is no installer yet: build cadatomic from source and run `cadatomic`.

A shortcut written "P then W" means press P, then W, with the pointer over the canvas. Page Down
zooms out at the pointer and Page Up zooms in. Undo is Ctrl+Z, and Ctrl means Cmd on macOS.

## 1. Start a project

A project is one folder for everything about one product.

With nothing open, the start page shows **New project…**, **Open project…** and **New part**.
Click **New project…**, or use **File ▸ New ▸ Project…**. In the folder dialog, make a new folder
called `led` and choose it. The project opens, named `led` after its folder.

## 2. Draw the three symbols

A symbol is a part's drawing on the schematic: a body and pins.

### Make the symbol library

1. Choose **File ▸ New ▸ New library**.
2. In the form that appears, type `led-symbols` in **Name**, press Tab, and click **Accept**.

The library goes into the project's `libraries/` folder and its tree opens. If **Name** is still
empty after you type, you have hit the blocker described at the top.

### Create a symbol and place its pins

Start with the LED. In the library tree, type `LED` in **New symbol** and `D` in **Prefix**, then
click **Create**. The prefix names placed copies: this LED will become D1. The empty symbol opens
with a small cross at its origin.

A new pin points right, and each press of Space turns it a quarter turn counter-clockwise: two
presses point it left, two more point it right again.

1. With the pointer on the symbol, press P then P for the pin tool.
2. Press Space twice so the pin points left.
3. Press Tab. The **Board properties** pane shows **Next pin**. Click its **Name** row, type
   `A`, and press Enter.
4. Click two grid squares left of the origin (−200, 0 mil). Pin 1 is placed.
5. Press Space twice and Tab, and name the pin `K`. Then click two squares right of the
   origin (200, 0).
6. Press Esc to end the pin tool.

Naming with Tab may not take in current builds. If a placed pin's name is still its number, click
**Pin table** above the symbol. It opens in its own **Pins** tab: type the name into that pin's
**Name** cell, press Enter, then click the symbol's tab to carry on.

### Set the value and draw the body

With nothing selected, **Board properties** shows the symbol itself. Set its **Value** to `LED`.
If the pane isn't showing, choose **View ▸ Open as tab ▸ Board properties**. Then draw the body:
P then Y for a polygon (click each corner, then Enter), P then L for a line, P then R
for a rectangle, and P then T for a text. A text reads `${REF}` unless you press Tab and set
**Text** to `${VALUE}` before you click. Esc ends each tool.

### The three symbols

Coordinates are in mil from the origin cross; negative y is below it.

| Symbol | Prefix | Value | Pins | Body |
|---|---|---|---|---|
| `LED` | D | LED | 1 `A` at (−200, 0) pointing left; 2 `K` at (200, 0) pointing right | triangle (−100, 100), (−100, −100), (100, 0); line (100, 100)–(100, −100); `${REF}` at (−100, 150); `${VALUE}` at (−100, −200) |
| `R` | R | R | 1 at (−200, 0) left; 2 at (200, 0) right | rectangle (−100, −50)–(100, 50); `${REF}` at (−100, 100); `${VALUE}` at (−100, −150) |
| `Conn_01x02` | J | Conn_01x02 | 1 at (−200, 0) left; 2 at (−200, −100) left | rectangle (−100, 50)–(0, −150); `${REF}` at (−100, 100); `${VALUE}` at (−100, −250) |

The resistor's and connector's pins need no names: a new pin's name is its number.

Press Ctrl+S to save the library, then **File ▸ Save project** so the project lists it.

## 3. Draw the three footprints

A footprint is the part's land pattern on the board. The LED's and resistor's pads come from
Kingbright's APT2012EC and Vishay's CRCW datasheets, and the header's hole from Samtec's TSW page.
The header's 1.7 mm land is a safe size above the IPC-2221 minimum.

Make a second library with **File ▸ New ▸ New library** again: name it `led-footprints`, press
Tab and click **Accept**. To start each footprint, type its name in **New footprint** in the
library tree and click **Create footprint**. It opens empty, and the canvas shows the pointer's X
and Y in mm.

For each footprint, in this order:

1. Click **Pad**, then press Tab: **Board properties** shows the next pad. Set
   **Width (mm)** and **Height (mm)**, then click where each pad goes. Pads number themselves. Esc
   ends the tool. For the header, set **Pad kind** to `th` first: **Hole** takes a value only then.
2. Click **Origin: centre**: the reference point moves to the pads' centre.
3. For the fab outline, click **Line**, then **Fab**, then the body's four corners and the first again.
   Esc twice ends the line, then the tool.
4. For the designator, click **Text**, then **Silkscreen**, then above the part. It reads REF until
   the part is on a board.
5. Click **Courtyard from pads…** (0.25 mm round pads and body), then **Accept
   courtyard**.

| Footprint | Pads (mm) | Pad centres (mm) | Fab box (mm) | Designator |
|---|---|---|---|---|
| `LED_0805` | 1.25 × 1.1 | (−1.175, 0), (1.175, 0) | corners ±1, ±0.625 | (0, 1.6) |
| `R_0805` | 0.95 × 1.45 | (−0.975, 0), (0.975, 0) | corners ±1, ±0.625 | (0, 1.6) |
| `PinHeader_1x02_P2.54mm` | Pad kind `th`, 1.7 × 1.7, Hole 1.02 | (0, 0), then (2.54, 0) | corners ±2.54, ±1.27 (after centring) | (0, 2.4) |

On the LED, pad 1 (left) is the cathode. Mark it after the fab outline: click **Line**, then
**Fab**, click (−0.6, −0.625) and (−0.6, 0.625), and press Esc twice.

The header needs extra moves. Before its pads, click **Grid** four times, to 1.27 mm. Place pad 1,
square, at the origin. Then set **Shape** to `circle` and click 2.54 mm right for pad 2, so that
pin 1 stands out. After the fab outline, click **Grid** once (back to 0.025 mm) and draw a
**Silkscreen** box just outside the body, corners ±2.625, ±1.325.

Press Ctrl+S, then **File ▸ Save project**. The project now lists both libraries.

## 4. Join symbols and footprints into parts

A part is what the schematic places: a symbol, its own copy of a footprint, and a pin-to-pad map.
The parts live in `led-symbols`.

1. In the library tree, select the `LED` symbol. A **New part from LED** form appears.
2. Set **Footprint library** to `led-footprints` and **Footprint** to `LED_0805`. Leave **Name** as
   `LED`. Click **Create part** (scroll the tree down if it is out of sight).
3. The part opens in its own tab. Type `LED` as its **Value** and press Tab.
4. Fix the pin map. Each pin's row has a toggle per pad, and a new part maps each pin to the pad
   of the same number. Pin 1 is the anode but pad 1 is the cathode, so on pin 1's row turn pad 1
   off and pad 2 on, and on pin 2's row the reverse.

Make the other two the same way. Part `R` comes from symbol `R` and `R_0805`, with value `R`, and
its map by number is right. Part `J` comes from symbol `Conn_01x02` and `PinHeader_1x02_P2.54mm`:
change **Name** to `J` and set the value to `Conn_01x02`.

Each part's footprint status reads **Up to date**: its copy matches `led-footprints`. Press
Ctrl+S.

## 5. Draw the circuit

1. Choose **File ▸ New ▸ New board design**. From the menu it asks no name: the design is called
   `untitled-1`, goes into the project's `designs/` folder, and its empty sheet opens. (From the
   command palette it asks for a name, in a form like New library's.)
2. With the pointer on the sheet, press P then P. In the strip at the bottom, open the
   **Part** list and choose `J` from `led-symbols`. Click to place it, then press Esc.
3. Press Page Down until there is room for two more parts on the left and a wire below them.
4. Press P then P, choose `R`, and click six grid squares left of the connector. Esc.
5. Press P then P and choose `LED`. Press Space twice to turn it half round, so its anode
   faces the resistor, then click six squares left of the resistor. Esc.
6. Press T then A to annotate: `J?`, `R?` and `D?` become J1, R1 and D1.

Now wire one loop through all three. Each wire joins the pins it touches into one net.

1. Press P then W to start the wire tool.
2. Click the end of J1's pin 1, then the end of R1's pin 2, and press Enter.
3. Click R1's pin 1, then D1's pin 1 (the anode), and press Enter.
4. Click D1's pin 2 (the cathode). Click on the empty sheet below D1 and again below J1 to make
   two corners, then click J1's pin 2 and press Enter. Press Esc to end the tool.

Last, the resistor's value. From 5 V the LED takes about 2 V, leaving 3 V across 330 Ω: about
9 mA. Click R1 and choose **View ▸ Open as tab ▸ Board properties**. Then double-click **Value**,
type `330 Ω` and press Enter.

## 6. Check the schematic

Click **Check (ERC)** at the top right of the sheet. Anything it finds is marked on the sheet: red
for errors, amber for warnings. Judge by these marks: in current builds the **Problems** tab may
not list the results of this check or of the board's.

It should mark nothing: every pin is passive and every net joins two pins. It cannot see a pin
with nothing on it at all, so look over the sheet too.

Press Ctrl+S with the pointer on the sheet, then **File ▸ Save project**.

## 7. Bring the circuit onto the board

The board never changes behind your back: schematic changes wait until you accept them.

Choose **View ▸ Open as tab ▸ Board**, then **View ▸ Open as tab ▸ Pending changes**. The
Pending changes tab lists the three parts and the three nets. Click **Acknowledge all**, and the
board takes the footprints and their nets. They lie off the board for now.

## 8. Shape the board and place the parts

If **Pending changes** now covers the board, choose **View ▸ Open as tab ▸ Board** again.

1. Press Ctrl+Page Down to fit the view on the three footprints.
2. With the pointer at the parts' left end, press Page Down three times to make room on the right.
3. Press D then S for the outline tool, and R once for 1 mm rounded corners.
4. Using the X and Y readout, click one corner at (26, −6) mm and the opposite one at (44, 6) mm.
5. At the board's top right, click **3 footprints off the board · Place them**. The footprints
   move into a column beside the outline.
6. Drag each part by its pad 1 onto the board's middle line: R1 to about (28, 0), J1 to about
   (33.7, 0) and D1 to about (39.8, 0). Close is fine.

Keep the courtyards, the lines round each footprint, apart. In this order no connection
crosses another, so every track fits on the top layer.

## 9. Route the tracks

The thin straight lines are the ratsnest: connections not yet made in copper.

1. Press P then T to route. Tracks go on the Top Layer at the default width.
2. Click R1's pad 2, then J1's pad 1.
3. Click J1's pad 2, then D1's pad 1, the LED's cathode.
4. Click D1's pad 2. Click above D1, at about (42.2, 4), then above R1, at about (28, 4): each
   click fixes a corner. Click R1's pad 1 to finish.
5. Press Esc to leave the tool.

No ratsnest line is left. If a corner goes wrong, you can press Backspace to take back the last
piece. A finished connection is one undo step.

## 10. Check the board and export

The design rule check runs by itself after every change and marks each violation on the board
with a circle and a cross. With no marker and no ratsnest line left, the board is clean.

1. Choose **File ▸ Export fabrication outputs…**. The **Fabrication outputs** pane opens.
2. Leave **Folder** empty and click **Export**.

The pane lists the files it wrote into the project's `outputs/untitled-1/fab`: Gerbers, a drill
file, a netlist, a job file, and `pick-place.csv` and `bom.csv` for assembly.

If Export refuses, its message says why (a board without an outline, for one), and nothing is
written.

Finish with **File ▸ Save project**. Your board is ready to send.
