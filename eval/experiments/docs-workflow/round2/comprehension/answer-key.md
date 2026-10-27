# Answer key: svelters + skelter comprehension questions (never shown to readers)

Paths are relative to `/home/user/svelters`. Every answer was checked against the code as built
(design docs cited only where the code matches them). Scoring: 2 = full, 1 = half, 0 = none.
Mix: mechanism Q1–Q5, why Q6–Q9, prediction/transfer Q10–Q12, debugging Q13–Q14.

---

## Q1 (mechanism): what compiles App.svelte, and when

**Reference answer.** With `--entry`, the runner builds a `SourceProvider` (the svelters module
loader) configured for the `skelter` backend and loads the app through it. When QuickJS asks for
the modules, starting from a generated entry that imports `App.svelte` and calls `mount(App)`, the
provider resolves imports Node-style and compiles each `.svelte`/`.svelte.js` file in-process with
the Rust svelters compiler. This happens at startup, inside the running binary, with no Node and no
Vite, and the output is cached by content. With a bundle, the compile already ran ahead of time in
the Vite build (`@svelters/build` invokes `svelters-compile`), and the runner only evaluates
`dist/app.js`.

**Sources.** `crates/svelters-skelter/src/bin/svelters-skelter-run.rs:55-63`;
`crates/svelters-modules/src/source/mod.rs:276-279` (compiles live, transform cache),
`:372-376` (generated entry `mount(App, …)`); `crates/svelters-modules/src/source/transform.rs:88-104`;
`crates/svelters-host/src/lib.rs:359-368` (`load_from`); `packages/build/index.js:10`.

**Rubric.** Full: (a) the svelters compiler (Rust) compiles it (b) live, in the runner's process, as
the module loader loads it at startup (c) contrasted with the bundle path, where the compile
happened at build time. Half: only "the svelters compiler compiles it to JS" without saying when or
where, or claims it happens in Vite/Node for `--entry`. Zero: "the browser/Svelte's JS compiler",
or a JIT per frame.

---

## Q2 (mechanism): from the click to `count++`

**Reference answer.** The platform delivers the input to the app handler on the UI (platform main)
thread. skelter-core runs the dispatch first: a hit test against the scene, then the native
defaults (press state, focus). Then, still inside the handling of that same input, the events it
produced are delivered to JavaScript (QuickJS on the same thread). Cancelable ones go through
synchronously so a listener can stop them or prevent the default. The closing host checkpoint drains
the microtasks, which is where Svelte's batched flush runs. Afterwards the app's sync step requests
a frame for any window whose tree changed.

**Sources.** `crates/svelters-skelter/src/app.rs:76-81` (dispatch: "native defaults, then JS at
event time"), `:173-206` (sync after every callback); `crates/svelters-skelter/src/deliver.rs:1-17`,
`:75-81`; `crates/svelters-host/src/core.rs:1062-1078` (checkpoint: task, then job drain);
`plans/research/skelter/architecture.md:631-645` (§11.1 order).

**Rubric.** Full: UI thread (same thread as the tree, not a worker); native hit test and defaults
before JS; handler runs at event time during the dispatch; microtask/flush drain after. Half: says
it runs on the UI thread at event time but omits native defaults first, or omits the drain/flush.
Zero: a separate JS thread, an event queue processed next frame, or a browser DOM event.

---

## Q3 (mechanism): what in the scene is updated when `count` changes

**Reference answer.** The label's text belongs to a template effect. When it re-runs, it makes one
host op, a `set_prop` of `text` on that label's handle. The backend maps it to
`Document::set_prop` on that one skelter-core node. Nothing is rebuilt: the button, the window and
any other nodes stay as they are. `text` is a Layout-class prop, so skelter-core marks only that
node to re-measure (relayout only if its intrinsic size changed), re-record its display list and
update accessibility, and bumps the revision. The next UI frame runs style, layout and recording
for the dirty part and sends a Commit to the compositor thread, which computes damage and draws
with wgpu.

**Sources.** `crates/svelters-compiler/tests/skelter.rs:79-95` (counter: the label's text is a
reactive `$n.set(…, "text", …)`); `crates/svelters-skelter/src/backend.rs:613-673`
(`write_prop_now` → `d.set_prop(n, pid, v)` at :659); `crates/skelter-core/src/document.rs:809-870`,
`:969-991` (`mark_class`); `crates/skelter-base/src/change.rs:6-15`;
`crates/svelters-skelter/src/app.rs:83-97` (frame → Commit); `plans/research/skelter/architecture.md:46-79`.

**Rubric.** Full: one prop (`text`) on one existing native node via a targeted set; no re-render of
siblings or the tree; the change class decides the work (measure/layout if needed, repaint/re-record);
the frame commits to the compositor, which draws. Half: "the label node's text prop is set" with no
account of how it reaches the screen, or a correct pipeline but the claim that the component or
window re-renders. Zero: virtual DOM diff, or the whole window is rebuilt.

---

## Q4 (mechanism): from `<window>` to an OS window

**Reference answer.** The compiled code creates a `window` node (a toplevel kind) and builds its
children under it while it is still detached. When the outermost build finishes, after the
fragment's template effect has written its props such as `title`, the runtime calls the window's
`present` method. The backend only records that. After the platform callback, the app's sync step
takes the pending presents, reads `title` and `default-width`/`default-height` (480×360 by
default), and asks the platform to open a window. When that window first reports its size, the
document attaches the JS-built node as the window's root (`attach_window`), and frames for it
begin.

**Sources.** `packages/native-runtime/src/index.js:431-476` (`showTop`/`building`/`presentTop`),
`:567-576`; `crates/svelters-skelter/src/backend.rs:965-973` (`present` recorded), `:757-783`
(`take_presents`, `window_desc`, `attach_presented`); `crates/svelters-skelter/src/app.rs:1-14`,
`:99-107` (first resize attaches), `:178-189` (opens windows).

**Rubric.** Full: JS builds the node tree first; `present` is called by the runtime once built; the
Rust app opens the platform window afterwards using the node's title/size props; the node is then
attached as that window's root. Half: "JS presents the window and Rust opens it" with no ordering,
or misses that the tree is built before the window exists. Zero: the window is created natively
first and Svelte mounts into it like a DOM target, or the OS window is created from the compiler.

---

## Q5 (mechanism): where `$state`/`$derived` run, and what Rust sees

**Reference answer.** They run in JavaScript inside the embedded QuickJS runtime, on Svelte's own
signal, effect and batching code: the native runtime imports stock Svelte 5's client internals
through one adapter module. Rust never sees signals, deriveds or component state. It sees only the
host operations the runtime's effects issue (create, insert, set_prop, listen, call…) on numbered
handles, and it sends events back in.

**Sources.** `packages/native-runtime/src/core/svelte.js:1-10` (only module importing
`svelte/internal/client`); `packages/native-runtime/src/index.js:1-6` ("Reactivity comes from the
core … backed by Svelte's signals … this module only turns component code into host ops");
`crates/svelters-skelter/src/backend.rs:833-935` (the `Backend` ops it receives); `CLAUDE.md`
("running stock svelte@5.57.2").

**Rubric.** Full: JS/QuickJS, Svelte's real reactivity runtime (not a reimplementation in Rust),
Rust sees only the resulting ops/handles plus delivers events. Half: says JS but believes Rust
tracks or mirrors the state, or says "a svelters reimplementation" without mentioning that Rust
only gets ops. Zero: reactivity is compiled to Rust, or skelter-core holds the signals.

---

## Q6 (why): no virtual DOM

**Reference answer.** Svelte compiles each dynamic expression into a fine-grained effect bound to
one prop of one already-created node, so when state changes the runtime knows exactly which writes
to make. There is nothing to diff. The native runtime keeps no JS tree of DOM-like nodes, only
handles. The retained scene lives in skelter-core. Each prop carries a change class (property,
paint, layout, structure), and a write marks exactly that work for that node. Damage comes from the
change journal, not from comparing render trees.

**Sources.** `packages/native-runtime/src/index.js:5-6`; `crates/skelter-core/src/document.rs:969-991`;
`crates/skelter-base/src/change.rs:6-15`; `plans/research/skelter/architecture.md:174-188` ("damage
comes from the journal (§7.4), not from diffing render trees").

**Rubric.** Full: compile-time knowledge of what depends on what, so targeted updates to retained
native nodes, plus the change classes / dirty marking as skelter's replacement for diffing. Half:
only one of the two halves (Svelte's fine-grained updates, or skelter's retained tree with dirty
marks). Zero: "for speed" with no mechanism, or a claim that a diff happens somewhere.

---

## Q7 (why): the compiler must know the backend

**Reference answer.** There is no DOM. Every element is a native kind of the target backend, with
that backend's props, their types, events, child slots and defaults. The compiler reads the
backend's semantic spec: for skelter, `skelter.json`, generated from svelters-skelter's Rust kind
tables and embedded in the compiler. It validates the template against the spec (an unknown tag,
prop or event is a compile error). It also shapes its output to the backend's capabilities:
skelter's spec turns on stable numeric ids (checked against the host at load) and native templates
(a static subtree built with one `instantiate`). A component compiled for GTK names other kinds and
props, so it would not run on skelter.

**Sources.** `crates/svelters-skelter/src/spec.rs:1-15`, `:27`
(capabilities `ids templates native_transitions native_motion css_precompiled`);
`crates/svelters-compiler/src/semantics/backend.rs:15`, `:55-63`;
`crates/svelters-modules/src/source/mod.rs:347-351`; `crates/svelters-compiler/src/codes.rs:73,80,84`;
`crates/svelters-compiler/tests/skelter.rs:41-95`; `packages/native-runtime/src/ids.js:1-10`.

**Rubric.** Full: elements/props/events come from a per-backend spec (generated from the Rust
backend), used both for validation and for backend-specific output (ids/templates or similar).
Half: only "to know which elements exist / validate", or only "to emit backend-specific code".
Zero: "for CSS" only, or "it doesn't really need to".

---

## Q8 (why): JS runs on the UI thread at event time

**Reference answer.** skelter's design runs JS where the tree lives, during the dispatch of the
input, for two reasons. A listener can cancel a cancelable event or prevent its native default
before that default takes effect. And JS's writes become part of the same transaction, so the very
next frame shows the result: no extra frame of latency from an outbox, and no cross-thread copy of
the tree. The cost is that a long handler blocks the UI thread: no input is handled and no new UI
frame is built until it returns. The host's turn budget is the backstop: in the runner every entry
into JS is interrupted after 10 s by default; an embed's default is 50 ms, which faults only that
root. The compositor runs on its own thread, so it can keep presenting what it already has.

**Sources.** `plans/research/skelter/architecture.md:68-79` ("JS runs on the UI thread at event
time"), `:631-645`; `crates/svelters-skelter/src/deliver.rs:1-17`;
`crates/svelters-skelter/src/embed/mod.rs:49-65` (budgets), `:97-104` (`EmbedConfig::app` uses the
host's `turn_budget`); `crates/svelters-host/src/core.rs:55-57`, `:104` (default 10 s).

**Rubric.** Full: a reason tied to the design (cancel/prevent the native default synchronously,
and/or the change lands in the same frame/transaction with no outbox lag) plus the cost (a slow
handler stalls the UI; the budget/interrupt as the guard). Exact budget numbers not required. Half:
only the reason or only the cost. Zero: "QuickJS can't do threads" as the whole answer, or a claim
that JS runs off-thread.

---

## Q9 (why): one font store, three consumers

**Reference answer.** Text is used in three places. The tree measures text to lay it out (the
backend's document, UI thread). skelter-widgets' kinds shape it into glyph runs (widget services).
The compositor rasterises the glyphs on its own thread (the glyph rasteriser). All three must
resolve the same fonts, or the boxes laid out from measured text would not match the glyphs drawn,
so they share one `FontStore` (its clones are the same store). A backend built without a text
system gets the `NoText` measurer, which measures every text as zero size. Labels and the button
would get no room for their text: collapsed or clipped, nothing legible.

**Sources.** `crates/svelters-skelter/src/bin/svelters-skelter-run.rs:48-51`, `:69-70`;
`crates/svelters-skelter/src/app.rs:35-49` (`install_widget_services`);
`crates/svelters-skelter/src/backend.rs:127-137` (`NoText` → `Size::ZERO`), `:292-301`.

**Rubric.** Full: names the three roles (measure for layout / shape / rasterise on the compositor),
the need for consistency (same fonts everywhere, so measured size equals drawn size), and the
zero-size consequence without a text system. Half: two of the three points. Zero: "fonts are
expensive to load" alone.

---

## Q10 (prediction): two windows, close one

**Reference answer.** Each `<window>` is presented separately, so two OS windows open, both
driven by the same component and the same `count`: clicking `+` updates both labels. Closing one
closes it at once. The skelter app handles the platform's close request directly. The backend frees
that window's node and reports its handle gone to JS (`SVL_GONE_CLOSED`), and the runtime marks the
node dead, so later writes to it are skipped. The other window, the component and `count` keep
running. The process exits only when the last presented window has closed (the runner's own rule;
the app is configured not to exit on a skelter-level last-window event). A JS `close-request`
listener is not consulted by this runner: the close goes straight to `close_window`.

**Sources.** `crates/svelters-skelter/src/app.rs:109-114`, `:158` (`exit_on_last_window: false`),
`:178-189`, `:209-211` (exit when `opened_any` and no windows); `crates/skelter/src/app.rs:300-313`,
`:867` (`CloseRequested => close_window`); `crates/svelters-skelter/src/backend.rs:461-468`
(`window_closed` → `on_destroyed(h, SVL_GONE_CLOSED)`); `packages/native-runtime/src/index.js:1414-1423`
(`gone`), `:677` (writes to dead nodes skipped).

**Rubric.** Full: two OS windows sharing one state; closing one leaves the other working and the
state intact; the process ends when the last window closes. Half: gets the two windows and the
"other stays" part but says the app exits on the first close, or says `count` resets / the
component unmounts. Zero: only one window appears, or both close together. (The `close-request`
detail is a bonus, not required.)

---

## Q11 (prediction): editing App.svelte while it runs

**Reference answer.** Nothing changes in the running app. With `--entry`, the source is compiled
once, at load. `svelters-skelter-run` never starts the host's file watcher (`Host::watch`), and
`--dev` only selects Svelte's dev mode and the `development` export condition; it does not enable
reloading. To see the edit you restart the runner. A host binary could call
`embed.host().watch()`, whose reload is a full reload in a fresh JS runtime (all JS state, including
`count`, is lost) unless the app is compiled for in-place HMR. (By contrast `svelters-egui-run` has
a `--watch` flag.)

**Sources.** `crates/svelters-skelter/src/bin/svelters-skelter-run.rs:1-74` (no watch call; `--dev`
sets `cfg.dev` only, :60); `crates/svelters-modules/src/source/mod.rs:63-66` (`dev`, `hmr`);
`crates/svelters-host/src/lib.rs:370-376`; `crates/svelters-host/src/reload.rs:1-22`;
`crates/svelters-egui/src/bin/svelters-egui-run.rs:16`, `:103`.

**Rubric.** Full: predicts no change, explains that compilation happened once at startup and the
skelter runner does not watch files (and `--dev` is not a watch flag). Half: predicts no change but
gives a wrong or no reason (e.g. "it's compiled to Rust"), or says "restart" without explaining.
Zero: predicts a hot reload or an in-place update.

---

## Q12 (prediction): an unchanged `$derived`, then idle

**Reference answer.** On the click from 3 to 4, `count` changes, so `size` is re-evaluated in JS,
but it is still `'few'`. Its dependents are not re-run, or if the fragment's shared template effect
re-runs for the count label, the runtime's last-value check skips the identical write. So no
`set_prop` for the `size` label crosses into Rust, and that label is not marked dirty: no measure,
no paint. Even a same-value write would be a no-op in skelter-core, which returns early when the
stored value is equal. Only the `Count:` label is updated. When you stop touching the app, it
renders zero frames: a window gets a frame only when the tree's revision moved or a frame was
requested, and idle means no frames and no timer wakeups.

**Sources.** `packages/native-runtime/src/index.js:695-698` (`set` skips a write equal to the last),
`:718-729` (`setp`); `crates/svelters-compiler/src/codegen/component.rs:710-714` (one template
effect per fragment); `crates/skelter-core/src/document.rs:836-839` (equal write returns early);
`crates/svelters-skelter/src/app.rs:95-97` (`needs_frame`), `:194-202`;
`plans/research/skelter/architecture.md:33` ("Idle: 0 frames, 0 timer wakeups").

**Rubric.** Full: derived recomputed but unchanged, so no op/no native work for that label (via
Svelte's equality and/or the runtime/skelter dedupe), and zero frames while idle. Half: one of the
two parts right (e.g. correct about no write but expects a steady 60 fps, or expects the `size`
label to be rewritten every click but gets idle right). Zero: expects every label re-rendered and
continuous frames.

---

## Q13 (debugging): `<input>` rejected at startup

**Reference answer.** There is no HTML or DOM. The only elements are the kinds in skelter's
semantic spec, and `input` is not one of them (the text field is `<text-edit>`). The svelters
compiler checks every tag, prop and event against that spec and reports `unknown_element` as an
error. With `--entry`, compilation happens while the module loader loads `App.svelte` at startup,
so the load fails with the rendered diagnostic. The runner prints it and exits with status 1 before
any window is presented.

**Sources.** `crates/svelters-compiler/src/codes.rs:73` (`unknown_element`: "not an element of the
backend's semantic spec"); `crates/svelters-modules/src/source/transform.rs:106-118` (errors stop
the load); `crates/svelters-skelter/src/bin/svelters-skelter-run.rs:65-67`;
`metadata/semantics/skelter.json` (`tags`: `text-edit`, no `input`).

**Rubric.** Full: elements must exist in the backend's spec (native kinds, not HTML), and it is
caught by the compiler during the live compile at load, before mount/window. Half: one of the two
(the "not in the spec" cause, or "compile time at startup"). Bonus, not required: names `text-edit`.
Zero: blames a missing import/runtime bug, or thinks it failed while rendering in the window.

---

## Q14 (debugging): "no skelter driver installed"

**Reference answer.** On skelter, the host does not own the event loop: the platform does. The
host is already attached when the embed is created (ABI `start`), and JS runs at the points the
platform calls into the app (input, timers, wakes, frames). `Host::run` hands the loop to the
backend's `Driver`, which a plain binary has not installed, so it fails. The binary should instead
create the compositor (`skelter::compositor`, with a glyph rasteriser) and hand an app handler
(`SveltersApp::new(embed, spawn)`) to `skelter_platform::run`, as the runner does. That handler
calls `embed.logic()` after every callback and opens the windows JS presents.

**Sources.** `crates/svelters-skelter/src/backend.rs:1112-1118` (`run` → "no skelter driver
installed …"); `crates/svelters-host/src/lib.rs:418-427` (`Host::run` calls the backend's `run`);
`crates/svelters-skelter/src/embed/mod.rs:387-392` (the embed `start`s the host),
`crates/svelters-skelter/src/app.rs:1-7`, `:173-176`;
`crates/svelters-skelter/src/bin/svelters-skelter-run.rs:69-73`.

**Rubric.** Full: identifies the inversion (the skelter platform/app runner drives the loop and
calls into the attached host; you don't call `host.run()`) and the fix (run the platform with the
svelters app handler/compositor, or install a driver). Half: the fix without the reason ("call
`skelter_platform::run` like the runner") or the reason without a workable fix. Zero: "install a
graphics driver"/GPU issue, or retrying with another flag.
