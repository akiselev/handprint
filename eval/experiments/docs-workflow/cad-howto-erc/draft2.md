# Check a schematic for wiring errors with ERC, then fix or waive what it finds

A schematic can look finished and still be wrong: a wire that stops just short of a pin, two
outputs on one net, an input left undriven. The electrical rules check (ERC) reads the circuit
your drawing makes and lists what looks like a mistake. Most of its checks and their default
levels come from Altium Designer's.

## Before you start

You need a design with a schematic open on its sheet, since the check runs on the active
schematic. You should also know the basic schematic tools: wire (P then W), net label (P then
N), power port (P then O), no-connect marker (P then V) and junction (P then J). These keys
need the pointer over the sheet.

Help ▸ **Tours…** has two guided tours on this subject: "Check the circuit (ERC)" and "Waive
an ERC violation, with a reason".

## Run the check

1. Click **Check (ERC)** at the top right of the sheet. The same command is in the command
   palette (Ctrl+Shift+P, or Cmd+Shift+P on macOS) as "Check the schematic (ERC)". It isn't
   bound to a key.
2. Wait for the job to finish. The bottom area opens on **Problems**, at its **Schematic**
   section, and the status line reads, for example, "Electrical rules check: 6 errors,
   0 warnings".
3. Look at the sheet. Each place the check found is marked with a ring and a cross: red for an
   error, amber for a warning.

A clean schematic gets a single row in the Schematic section: "Electrical rules check: no
violations".

You don't need to press the button again. Once it has run, the check follows the schematic
until you close it. Every edit, undo and redo reruns it in the background, and a violation you
fix drops off the list by itself.

## Read a violation

Each row shows the level (Error or Warning), the check's name, such as "Nets with only one
pin", and where it is: the schematic, the sheet and the position in millimetres. A finding
without a place shows the schematic alone. Errors come before warnings.

1. Click a row. The sheet switches to the right page, zooms to frame the place and selects the
   items there.
2. To trace the net involved, hold Ctrl (Cmd on macOS) and click one of its pins. The whole net
   lights up on the sheet and, if the design has a board open, on the board.

Some findings, such as "Nets with multiple names", "Duplicate part designators" or "Drawing
disagrees with the circuit", are about the design as a whole rather than one spot. They are
listed without a marker on the sheet, and clicking their row leaves the view where it was.

One mistake can fire several checks. An unwired input pin can show up as "Unconnected objects
in net", "Nets with no driving source" and "Nets containing floating input pins" at once; fix
the cause and all three clear.

## What the common findings mean

The level in the table is the default. A design imported from an Altium project keeps that
project's levels, and those win. This version of the app has no settings page for changing them.

| Problems row | Default level | What it means | Usual fix |
|---|---|---|---|
| Nets with only one pin | Error | A pin has a wire, label or other object on it, but it is the net's only pin: a connection started and not finished. | Finish the wire, or delete the stub. |
| Unconnected objects in net | Warning (from the connection matrix) | A pin or port stands on a net by itself. By default the check reports input pins, input ports and sheet entries, and no other kind of pin or port. | Wire it, or give an open pin a no-connect marker. |
| Nets containing floating input pins | Error | Every pin on the net is an input and the net lacks a power port, so its level floats. | Connect a driving output, or a power port. |
| Nets with no driving source | Warning | The net lacks a driver: an output, IO, open-collector, open-emitter, HiZ or power pin, an input or bidirectional port, or an output or bidirectional sheet entry. A net made entirely of passive items is exempt. | Connect a driver (see power ports below). |
| Nets containing multiple similar objects | Error for two output pins | Two items of a kind that should not share a net, most often two outputs. | Separate the nets, or check the pin types. |
| Nets with possible connection problems | Error for output + power or output + open collector | Two kinds of pin that conflict, such as an output pin wired to a power pin. | Separate them, or check the pin types. |
| Nets with multiple names | Warning | One net carries two names: two different net labels, or a label and a power port. | Delete or rename one. |
| Floating net labels | Warning | A net label stands apart from every wire, pin, port and power port, so it names nothing. | Drag it onto its wire, or delete it. |
| Duplicate part designators | Error | Two placed parts share a designator (compared ignoring case). Two parts still called R? count. | Annotate, or rename one part. |
| Drawing disagrees with the circuit | Error | A drawn group joins two nets, or a pin touches a wire of a net it is not on. | The row has no place to jump to. Look for a wire end on a pin of another net, or a group joining two nets, and redraw it. |

## Fix the common problems

### A net with only one pin

Usually a wire ends near a pin but not on its end, or two wires cross without joining.

1. Click the row to go to the pin.
2. Click an empty spot of the sheet, then zoom out until you can see where the wire should go.
   Page Down zooms out at the pointer.
3. With the pointer on the sheet, press P then W, click the end of the pin, click the end of
   the other pin, then press Enter and Esc.

The check reruns. If both pins were alone on their nets before, both errors go at once.

Wires that cross without one ending on the other are not connected. To join them, press P then
J and click the crossing to place a junction.

If the pin is meant to stay open, delete the stub wire and give the pin a no-connect marker, as
the next section describes.

### A pin that is meant to stay open

An unused input of a spare gate, or a pin the datasheet says to leave floating, should carry a
no-connect marker; the check then ignores that pin.

1. With the pointer on the sheet, press P then V.
2. Click the end of the pin. A small cross appears on it.
3. Press Esc.

The check does not report a passive pin, such as a resistor's, that has no wire, so it
will not catch a part you placed and forgot to wire.

### Undriven inputs and missing power

"Nets containing floating input pins" means a net made entirely of inputs. Connect the output
that should drive it. If the input should sit at a fixed level, tie it to a supply instead, and
use a supply name other parts are already on; otherwise the input is left as a single-pin net.

1. With the pointer on the sheet, press P then O. The power port tool starts on GND; change its
   **Name** in the tool strip if you need another supply.
2. Click the end of the pin, then press Esc.

All power ports with one name are one net, on every sheet. A power port clears "Nets
containing floating input pins" but is not a driver for "Nets with no driving source". That
warning clears when a driver is on the net somewhere in the design, such as a regulator's power
pin on the same supply name.

### Conflicting outputs on one net

Two output pins ("Nets containing multiple similar objects"), or an output and a power pin
("Nets with possible connection problems"), would drive the net against each other. Find the
wire that joins them and remove or reroute it. If the connection is right and a pin type is
wrong, say an open-collector output drawn as a plain output, fix the pin's **Electrical type**
in the part's symbol in the library. The design keeps its own copy of each part, so then open
**Components library** from the command palette and click **Update** beside the part, which now
reads "Out of date".

### Net names and labels

For "Nets with multiple names", keep the name you want and delete or rename the other label or
port. Two labels with the same name connect their nets without a wire, so a stray label can
merge nets you meant to keep apart.

### Duplicate designators

If the parts are still called R?, C? or U?, press T then A with the pointer on the sheet to
annotate the schematic. Every such part gets the next free number, and locked designators stay.

If two parts already have the same number, select one and open **Properties pane** from the
command palette. Double-click **Designator**, select its text (Ctrl+A, or Cmd+A on macOS), type
the new one and press Enter. The pane opens as a tab over the sheet; click the sheet's tab to go
back.

## Waive a finding you accept

Sometimes the check is right by its rules and the circuit is still what you meant, such as a pin
left open for a test probe. You can waive that one finding and write down why.

1. Right-click the row in Problems. The Menu key or Shift+F10 opens the same menu on the
   selected row.
2. Choose **Waive…**. The row turns into a text field that asks "Why is it acceptable?".
3. Type the reason and press Enter, or click **Waive**. Esc or **Cancel** leaves without
   waiving.

A No ERC directive is added to the design at the violation's place. It records your reason, your
name (your login name on the computer) and today's date. The reason is saved with the design,
but this version of the app doesn't show it in the list. The row stays in Problems, now reading
**Waived** instead of Error or Warning. Its marker leaves the sheet, and it drops out of the
error and warning totals.

A waiver covers one kind of finding at one place. Other checks at the same spot still report,
and if a later edit produces a new violation of the same kind at that place, the waiver covers
that too.

A finding without a place on the sheet (multiple names, duplicate designators, drawing disagrees
with the circuit) can't be waived: its menu lacks **Waive…**. Fix those instead.

## Remove a waiver, or undo one

Right-click the waived row and choose **Remove waiver**. The No ERC directive is deleted and the
finding counts again. Waiving and removing a waiver are ordinary edits, so Ctrl+Z (Cmd+Z on
macOS) takes either one back, reason and all.

## When something goes wrong

If the check misses something you expected, make sure the right schematic is active: the check
runs on the one you last worked in.

If Problems shows "The electrical rules check could not read the schematic", the design could
not be turned into a circuit, so the check has produced no results. If it appeared after an
edit, undo that edit; the check reruns by itself.

If **Waive…** closed without waiving, the reason was empty. Choose **Waive…** again and type
one.

## Gaps in this version

The list doesn't show the full message, which names the net and pins (for example "Net N0 has
only one pin (R1-1)"), or a waiver's reason, author and date. The No ERC directive is not drawn
on the sheet.

## Check from the command line

Copperfield, the ECAD engine under cadatomic, has its own command-line tool, which isn't
installed with cadatomic. `copperfield erc` runs the same checks on a saved design folder or an
Altium project or schematic. It prints each violation with its full message, or just the counts
with `--summary`, and exits with status 1 when any unwaived violation is an error:

```
copperfield erc [--summary] <design-dir | file.PrjPcb | file.SchDoc>...
```
