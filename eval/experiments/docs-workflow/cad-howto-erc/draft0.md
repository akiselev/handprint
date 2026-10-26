# Check a schematic for wiring errors with ERC, then fix or waive what it finds

A schematic can look finished and still be wrong: a wire that stops just short of a pin, two
outputs fighting over one net, an input that nothing drives. The electrical rules check (ERC)
reads the circuit your drawing makes (which pins, wires, labels and ports form each net) and
lists what looks like a mistake. Its checks and default levels follow Altium Designer's.

## Before you start

- A design with a schematic open in a sheet pane. The check runs on the active schematic.
- The basic schematic tools: wire (**P** then **W**), net label (**P** then **N**), power port
  (**P** then **O**), no-connect marker (**P** then **V**) and junction (**P** then **J**).

If you would rather be walked through it, Help ▸ **Tours…** has two guided tours on this page's
subject: "Check the circuit (ERC)" and "Waive an ERC violation, with a reason".

## Run the check

1. Click **Check (ERC)** at the top right of the sheet. (From the command palette,
   Ctrl+Shift+P, or Cmd+Shift+P on macOS, the same command is "Check the schematic (ERC)".
   It has no keyboard shortcut of its own.)
2. Wait for the job to finish. The bottom area opens on **Problems**, at its **Schematic**
   section, and the status line reads, for example, "Electrical rules check: 6 errors,
   0 warnings".
3. Look at the sheet. Each place the check found is marked with a ring and a cross: red for an
   error, amber for a warning.

If nothing is wrong, the Schematic section holds one row: "Electrical rules check: no
violations".

You don't need to press the button again. Once it has run, the check follows the schematic: every edit, undo and redo reruns it in the background, and a violation you fix
disappears from the list by itself.

## Read a violation

Each row in the Schematic section shows three things: the level (**Error** or **Warning**),
the name of the check that fired, such as "Nets with only one pin", and where it is: the
schematic, the sheet and the position in millimetres. Errors are listed before warnings.

1. Click a row. The sheet switches to the right page, zooms to frame the place and selects the
   items there.
2. To trace the net involved, hold Ctrl and click one of its pins. The whole net lights up, on
   the sheet and on the board.

Some findings, such as "Nets with multiple names" or "Duplicate part designators", are about the
design as a whole rather than one spot. They have no marker on the sheet and are only listed.

One mistake can fire more than one check. An input pin left completely unwired, for example,
can show up as "Unconnected objects in net", "Nets with no driving source" and "Nets containing
floating input pins" at once. Fix the cause and all of them clear together.

## What the common findings mean

The level in the table is the default. A design can carry its own levels (an Altium project
does), and those win; this version of the app has no settings page for changing them.

| Problems row | Default level | What it means | Usual fix |
|---|---|---|---|
| Nets with only one pin | Error | A pin has a wire, label or other object on it, but no second pin shares the net. A connection was started and not finished. | Finish the wire to the pin it was meant for, or delete the stub. |
| Unconnected objects in net | Set by the connection matrix | A pin or port stands on a net by itself. By default only input pins, input ports and sheet entries are reported. | Wire it, or mark the pin with a no-connect marker if it is meant to stay open. |
| Nets containing floating input pins | Error | Every pin on the net is an input, and no power port is on it: nothing sets its level. | Connect an output that drives it, or tie it to a supply with a power port. |
| Nets with no driving source | Warning | The net has no pin that drives it (output, IO, open collector, open emitter, HiZ or power pin). | Connect a driver. See the note on power ports below. |
| Nets containing multiple similar objects | Error for two output pins | Two items of the same kind that should not share a net, most often two outputs. | Separate the nets, or check the pin types. |
| Nets with possible connection problems | Error for output + power or output + open collector | Two kinds of pin that conflict, such as an output pin wired to a power pin. | Separate them, or check the pin types. |
| Nets with multiple names | Warning | One net carries two names: two different net labels, or a label and a power port. | Delete or rename one so the net has a single name. |
| Floating net labels | Warning | A net label touches no wire, pin, port or power port, so it names nothing. | Move it onto the wire it was meant for, or delete it. |
| Duplicate part designators | Error | Two placed parts share a designator (compared ignoring case). Two parts still called R? count. | Annotate, or rename one part. |
| Drawing disagrees with the circuit | Error | What is drawn and the circuit don't match: a drawn group joins two nets, or a pin touches a wire of a net it is not on. | Go to the place and redraw the connection. |

## Fix the common problems

### A net with only one pin

This is the error you will see most. Usually a wire ends near a pin but not on its end, or two
wires cross without joining.

1. Click the row to go to the pin.
2. Zoom out until you can see where the wire should go (Page Down zooms out at the pointer).
3. Press **P** then **W**, click the end of the pin, click the end of the other pin, then press
   Enter and Esc.

The check reruns. If both pins were alone on their nets before, both errors go at once.

If the wire you expected to connect crosses another wire instead of ending on it, the two are
not connected: a schematic connects wires only where one ends on another. Press **P** then
**J** and click the crossing to place a junction.

If the pin is meant to stay open, delete the stub wire and give the pin a no-connect marker
(next section).

### A pin that is meant to stay open

An unused input of a spare gate, or a pin the datasheet says to leave floating, should carry a
no-connect marker. Pins under one are left out of the check entirely.

1. Press **P** then **V**.
2. Click the end of the pin. A small cross appears on it.
3. Press Esc.

Note that the check does not report a passive pin (a resistor's or capacitor's, say) that has no
wire at all, so it will not catch a part you placed and forgot to wire.

### Inputs with nothing driving them, and missing power

"Nets containing floating input pins" means a net made only of inputs. Connect the output that
should drive it, or, if the input should sit at a fixed level, tie it to a supply:

1. Press **P** then **O**. The power port tool starts on GND; change the name in the strip if you
   need another supply.
2. Click the end of the pin, then press Esc.

All power ports with the same name are one net, on every sheet. A power port on the net clears
"Nets containing floating input pins". It does not count as a driver for "Nets with no driving
source", though: that warning clears only when a driving pin is on the net somewhere in the
design, such as the power pin of a regulator connected to the same supply name.

### Two outputs on one net

"Nets containing multiple similar objects" with two output pins, or "Nets with possible
connection problems" with an output and a power pin, means two pins would drive the net against
each other. Find the wire that joins them and remove or reroute it. If the connection is
correct and the pin types are wrong (an open-collector output drawn as a plain output, say), fix
the pin's **Electrical type** in the part's symbol in the library.

### Net names and labels

For "Nets with multiple names", decide which name the net should have and delete or rename the
other label or port. Remember that two labels with the same name on a sheet connect their nets without a
wire, so a stray label can merge nets you meant to keep apart.

For "Floating net labels", drag the label so it touches the wire, or delete it.

### Duplicate designators

If the parts are still called R?, C? or U?, press **T** then **A** to annotate the schematic:
every such part gets the next free number, and locked designators stay. If two parts already
have the same number, select one, open the **Properties pane** from the command palette,
double-click **Designator**, type a new one and press Enter.

## Waive a finding you accept

Sometimes the check is right by its rules and the circuit is still what you meant: a pin left
open for a test probe, a connection made somewhere the check can't see. Instead of turning the
check off, you waive that one finding and write down why.

1. Right-click the row in Problems. (The Menu key or Shift+F10 opens the same menu on the
   selected row.)
2. Choose **Waive…**. The row turns into a text field that asks "Why is it acceptable?".
3. Type the reason and press Enter (or click **Waive**). Esc or **Cancel** leaves without
   waiving.

What happens:

- A No ERC directive is added to the design at the violation's place. It records your reason,
  your name (your login name on the computer) and today's date.
- The row stays in Problems, now reading **Waived** instead of Error or Warning, so the next
  person to read the design sees it.
- Its marker leaves the sheet, and it no longer counts in the error and warning totals.

A waiver covers one kind of finding at one place. Other checks at the same spot still report,
and if a later edit produces a new violation of the same kind at that place, the waiver covers
that too.

Not every row can be waived. A finding with no place on the sheet (multiple names, duplicate
designators, drawing disagrees with the circuit) has no **Waive…** item; fix those instead.

## Remove a waiver, or undo one

1. Right-click the waived row.
2. Choose **Remove waiver**.

The No ERC directive is deleted and the finding counts again. Waiving and removing a waiver are
ordinary edits, so Ctrl+Z (Cmd+Z on macOS) takes either one back, reason and all.

## When something goes wrong

- **The check reports nothing you expected.** Make sure the right schematic is active: the check
  runs on the one you last worked in. Remember too that unwired passive pins are not reported.
- **"The electrical rules check could not read the schematic".** The design could not be turned
  into a circuit, so there are no results. Undo your last change and run the check again.
- **Waive… did nothing.** The reason was empty. A waiver needs one; choose **Waive…** again and
  type it.

## Gaps in this version

- The Problems row shows the check's name and place, not the kernel's sentence with the net and
  pin names ("Net N0 has only one pin (R1-1)") or the waiver's reason, author and date. Both are
  kept with the row's data, but the list does not draw them yet; select the row to see the place.
- The No ERC directive is not drawn on the sheet.
- There is no page for changing the levels or the connection matrix.

## Check from the command line

If you have Copperfield's command-line tool, `copperfield erc` runs the same checks on a saved
design folder. It prints a count per check, then each violation with its level, marking waived
ones "(waived)", and exits with status 1 when any unwaived violation is an error:

```
copperfield erc [--summary] <design-dir | file.PrjPcb | file.SchDoc>...
```
