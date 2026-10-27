# Reviser log: draft0 → draft1

## Verify issues

1. Compiler output (ids): **fixed.** The section now prints the real 28-line `App.js` and walks it top to bottom. Props travel by name (`$n.set(…, "text", …)`). One paragraph says what stable ids are for (skip the per-call name lookup, check the spec once per module) and that the mode is built and listed but doesn't switch on for skelter today. No bug detail and no internal ids beyond the reader-checkable `135398928` search.
2. Error message column and truncation: **fixed.** The page shows the observed full message from the short path `/tmp/hs2/app` (column `3:6`), says why the column is 6, and describes the observed long-path cut. It also says where the cut falls for a `/home/you/svelters` checkout (arithmetic on the 255-byte rule) and shows the CLI's untruncated output.
3. Hover "visibly changes": **fixed.** It now says what the reader sees: in current builds the hover fill doesn't follow the pointer, appears only after a click and stays after the pointer leaves. That's called a probable repaint bug, not a design. The counter prediction is unchanged (it held).
4. "Button and box not touched": **fixed.** `count++` doesn't touch them, but the button repaints for its own press and hover state, which Rust handled before JS ran. The reader is pointed at the lighter fill after a click.
5. Logger noise: **fixed.** The real block is shown (two winit WARNs, two EGL ERRORs, the INFO line, the traffic line), and each line is explained (Xvfb, wgpu probing GL, Vulkan chosen). The narrower filter is offered as an option.
6. Silent black window, second cause: **fixed.** "No working GPU path: no adapter at all, or a GL driver that doesn't draw". Added the verifier's `VK_ICD_FILENAMES=/nonexistent.json` run as a short experiment, with the observed `no GPU adapter` line, and said plainly that the GL-works case wasn't tried.
7. "Platform speaks first": **fixed.** The failure list is reordered (loader/compiler, JS, platform, compositor) and explained by the startup order. Added the verifier's no-display run as an experiment, with its observed output order.
8. "Only through events": **fixed.** The model section, diagram and paragraph now name two return paths, events and change notices. The `changes` counter is explained in the traffic section and tied back in "Where Svelte stops".
9. `name` derived claim: **fixed** with the verifier's working Rename version and its observed log. Added a compile-and-compare step: with Rename the thunk is `$.get(name)`; without it, `name` is a plain variable. This turns draft0's mistake into a lesson about what the compiler makes a signal.
10. Missing startup lines: **fixed.** The three mount lines are shown before the click block and explained (template effect before `$effect`, same order as a click).
11. Who says `Load:` / the 256 cut: **fixed.** `Load:` is svelters-host, "Error loading module" is the engine binding, and QuickJS's fixed 256-byte buffer cuts after 255 bytes.
12. CLI needs its own build: **fixed.** "The first run builds the CLI, which takes under half a minute" (observed 23.8 s).
13. Light theme: **fixed.** It says the snippet names Linux where `install_widget_services` picks the OS, tells the reader to delete the `<style>` block, and says to restore the theme (and clear colour) afterwards.
14. Unexplained counters and classes: **fixed.** The four change classes are named with the code's own examples where the change class is introduced, with a pointer to grep `"change": "paint"`. Every Traffic field gets a reading: the JS→Rust half is matched number by number to the compiled code, and `syncs`, `posts`, `changes`, `frames` and `checkpoints` are explained.
15. Black-window experiment: **fixed.** Added the clear-colour `spawn` replacement with a prediction prompt and the observed result (navy background, button and text unchanged), plus the `None` meaning from the field's doc. It is placed after the light theme, the state it was run in.

None rejected.

Also fixed from the verifier's notes: build time "two to three minutes" (2:07 and 2:44 observed). The dump experiment now tells the reader to restore the plain counter first, since the dump shown is of that app. The `[log]` line before the dump is explained. Cargo.lock gains the package. `env -u DISPLAY` uses the runner, as observed. Wayland, a real GPU and a working-GL machine are marked not tried. The libxkbcommon-x11 panic and "cannot open window" are kept as round-1 observations at the same commit, phrased as "on a machine without …".

## Teaching changes

- Concept check (concepts.md 1–8). Each idea is introduced, used and seen in an experiment. 1: the startup steps, the clear-colour change to the compositor, the logger's compositor line. 2: break-it. 3: compiled JS and the creates/tree/sets/listens counts. 4: click path and posts. 5: Rename log, plain `{name}` sets and change classes. 6: hover traffic and the "Where Svelte stops" counts. 7: three experiments, one per fact. 8: no-display and no-adapter runs. Goals 3 and 8 were reworded in concepts.md (props by name, change notices, failure order).
- New "why" content: what `$.get` is for (dependency tracking), why static props sit outside effects, how to read the template table, why mount and click share one effect order, and why a never-assigned `$state` compiles to nothing.
- Frame-side claims the reader can't observe are now flagged once ("you can't watch the frame side…"), with the source files named. No new unverifiable assertions were added.
- Every "you should see" is replaced by the observed output.
- Style. Removed bold from step lead-ins, list lead-ins and emphasis; bold stays for defined terms and UI labels. Broke up prose semicolon chains. handprint now flags neither bold nor semicolons. Not acted on, as instructed: the negation flag (the "there is no X, instead Y" contrasts) and the ellipsis, emoji and opener flags (mostly inside code spans and output).
- Length: about 6,600 words of prose, against 4,660 in draft0 and the brief's rough 3,000–5,000. The growth is the requested additions (full compiled-JS walkthrough, clear-colour, Rename, no-display and no-adapter experiments, counter readings, real logger output). If length must come down, the cadatomic paragraph and the hosting read-through are the safest cuts.

## For the coordinator (svelters bugs seen, not in the page)

- skelter.json nests `spec_hash` under `"ids"`, but the compiler reads it from the top level, so stable ids never turn on for skelter (`crates/svelters-skelter/src/spec.rs:400`, `crates/svelters-compiler/src/semantics/spec.rs:62`, `compile.rs:261-264`; `tests/spec.rs:47` pins the nested form).
- The button's hover fill doesn't repaint on pointer moves alone. It appears after a press and sticks.

## Pass 3 — trim (draft1 → draft2)

Prose outside code blocks: 6,591 → 4,952 words (same count method as before). No new claims, facts or outputs.
Every code block and output left in draft2 is one from verify.md, or an excerpt of one (`runs/App.js`).

Cut, as the two blind readers asked:
- **Numeric prop-id digression** (the `135398928` search, the ids mode, the spec-hash check) — padding.
- **"Less than the spec allows" templates aside**, and the matching "Templates aren't native yet" bullet,
  which pointed at a passage no longer on the page — padding.
- **Logger section**: the `StderrLog` code, `set_logger` line, the WARN/ERROR/INFO log block and its
  line-by-line reading, and the whole `VK_ICD_FILENAMES` experiment — padding / attention drop. One sentence
  keeps the useful fact: the compositor logs through `log`, and with a logger installed the silent black
  window names its cause (`no GPU adapter for backends …`, observed).
- **Compiled-output walkthrough**: the full 28-line `App.js` became a 12-line excerpt of the same file
  (`$.state`/`$.derived` with `$.get`, the window's static `$n.set` lines, the `$.template_effect`). Kept:
  `$.get` as the dependency tracker, static props outside effects, the one template effect that re-runs, no
  HTML/render/virtual node. Dropped: the `$$T0` table reading, `$n.at`, `$n.slot`, `$n.instantiate`,
  `$n.template_listen`, `$.push`/`$.pop`, `[() => 8]`. The table survives as one parenthetical.
- **Break-it**: the long-path paragraph shrank to three sentences; the CLI's own copy of the diagnostic was
  dropped (the CLI is introduced in the compiler section, and the failures list says to use it on a cut
  message).
- **Hover-repaint aside** → one parenthetical sentence after the counter table.
- **Black window**: the three facts and the clear-colour experiment kept whole; the panel and theme steps
  compressed (the `<style>` and theme code unchanged, the "that is the point" sentence and the OS-picking
  aside cut).
- **Failures**: the four-layer list kept as a compact list, one item per layer with its observed message;
  node_modules alias detail and the GL-works speculation cut.
- **Hosting read-through**: five bullets kept, each roughly halved (`cfg.backend` sentence dropped).
- Smaller compressions: startup steps 1, 3–5; kinds paragraph; change-notice paragraph; extension points
  and the cadatomic paragraph (`use:op` and "docking and tabs" dropped); "Where Svelte stops"; the opening
  scope paragraph (no-release is said once, in "What isn't there yet"); the duplicate "Linux and X11 only"
  bullet.

Restorations restructured. draft1 had the reader put things back four times (the `<label>` after break-it;
the plain counter before the tree dump; the `{name}` label; the theme and clear colour). Now:
- The runner experiments (break-it, `onfoo`, the event-order logging, Rename and its compile check) all go in
  one scratch copy at `/tmp/hs2/app`, the short path the break-it output was observed from, so the real app
  only moves forward: hello → counter → own binary.
- The tree dump therefore needs no "put the counter back".
- The theme and clear-colour changes are the last code on the page (the logger is gone), so nothing asks to
  undo them.
- One revert remains: taking the plain `{name}` label out before the black-window section, whose runs were
  observed without it. Deleting `<style>` before the theme step stays, as a step of that experiment.
- Checked by re-running the moved steps from a copy outside the checkout (`/tmp/hs3/app`, the verifier's
  binaries at `6c1ea458`, Xvfb): the hello window opens; `<push-button>` gives the same `3:6` diagnostic from
  the runner and the CLI; the logging + Rename variant with one Increment and one Rename click gives exactly
  the observed ten lines. (The verifier's own Rename run, `runs/app-name2`, was already from outside the
  checkout.)

Scaffolding and tells:
- "Predict … then run" kept in two places, where the answer surprises: whether Rename re-logs `count` and
  `doubled`, and which parts change with the clear colour. The event-order, panel and counter predictions are
  now plain statements ("The click path says what the other two runs should show").
- "That is the point", "That is what 'fine-grained' means here" and a few tidy triads cut.
- The tree-dump helper now says in one clause that it reaches into undocumented backend internals and should
  be copied as is.

Kept untouched, as the readers asked: the opening model and diagram, the click path, the four change
classes, the traffic counters and their table, the break-it experiment, the clear-colour experiment.

Not done: splitting the last third into its own page (the trim brought the page into range without it).
