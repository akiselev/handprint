# Facts: sv-tutorial (round 2, "how svelters + skelter work")

svelters checkout `6c1ea458` (/home/user/svelters), the same commit round 1 was verified on. cadatomic `b350c24`
(/home/user/cadatomic), read for one paragraph. Paths are relative to the svelters checkout unless marked
`cadatomic:`. Nothing was built or run by this writer. Revised for draft1 (2026-10-27): every observed fact now
cites `verify.md` (round 2, a run of draft0 on `6c1ea458` under Xvfb + lavapipe) as "verify.md execution log,
<section>"; entries the run contradicted are marked **superseded** and point at section J.

Grades:
- **code** — read in the source at the cited line.
- **observed** — round 1's real run (`exp/sv-tutorial-first-skelter-app/verify.md`, Ubuntu 24.04, Xvfb, Mesa
  25.2.8, lavapipe), cited as `verify:<step/issue>`.
- **design** — a design document's *why*; each one used is checked against the code named next to it.
- **inferred** — reasoned from the cited code; not run. Predictions in the draft's experiments are marked P*.

## A. The process and its pieces

- A1 (code) The runner's `main` builds, in order: a `FontStore` over system fonts; `SkelterBackend::with_text(builtin_kinds(), TextSystem)`;
  `install_widget_services`; `HostConfig::default()`; `Embed::with_backend(backend, EmbedConfig::app(host))`;
  with `--entry`: `SourceConfig::for_path`, `cfg.backend = "skelter"`, `SourceProvider::new`, `embed.load_app_from`;
  then `FontRasterizer`, `skelter::compositor(... Span::from_micros(16_667) ...)`, `skelter_platform::run(Box::new(SveltersApp::new(embed, spawn)))`.
  — `crates/svelters-skelter/src/bin/svelters-skelter-run.rs:48-73`
- A2 (code) "The app mounts its `<window>`s itself; each one it presents opens a platform window (winit) drawn by
  skelter's compositor (wgpu)." — `svelters-skelter-run.rs:8-9`
- A3 (code) `SkelterBackend` implements the backend ABI on one `skelter_core::Document`: "Every op is a call on the one
  Document with typed, by-id props". — `crates/svelters-skelter/src/backend.rs:1-4`
- A4 (code) `SkelterBackend::with_text` creates the `Document` on the kinds' registry. — `backend.rs:298-301`
- A5 (code) svelters-host "runs an ESM bundle in embedded QuickJS-NG and forwards its host ops to a `svl_backend`";
  "a `Host` is pinned to the thread that created it (the toolkit's main thread). The host starts no threads."
  — `crates/svelters-host/src/lib.rs:1-2`, `:18-19`
- A6 (code) QuickJS comes from `rquickjs` 0.14.0. — `Cargo.toml:90` (round 1 F46)
- A7 (code) `Embed::with_backend` makes the `Host` over the backend and starts it (attached: the platform owns the loop).
  — `crates/svelters-skelter/src/embed/mod.rs:389-403`; `app.rs:5-7`
- A8 (code) `skelter_platform::run` runs a winit event loop on the calling (main) thread. — `crates/skelter-platform/src/backend.rs:44-58`
- A9 (code) "One compositor thread per process owns the wgpu Instance, Device and Queue, every surface ... It never calls
  winit or the OS window API." The thread is spawned in `Compositor::spawn`. — `crates/skelter-compositor/src/lib.rs:3-7`, `:116`, `:150-152`
- A10 (code) The skelter facade's `App` "runs on the UI thread as the platform's app handler: it owns the tree,
  dispatches input at event time, answers the compositor's `BeginFrame`s with commits". — `crates/skelter/src/lib.rs:3-5`
- A11 (code) `SveltersApp` opens a window for each `<window>` JS presents: JS builds a `window` node and calls `present`;
  after the callback the app asks the platform for a window with the node's title and `default-width`/`default-height`;
  the window's first size attaches the JS-built node as its root. — `crates/svelters-skelter/src/app.rs:1-15`, `:173-188`, `:99-107`
- A12 (code) Until then the presented window "is a detached tree" (comment on `present`). — `backend.rs:966-974`
- A13 (code) `window_desc` reads `title`, `default-width` (480), `default-height` (360) from the node's props. — `backend.rs:763-774`
- A14 (code) The app exits when the backend quits or when it has opened a window and none remain. — `app.rs:209-211`
- A15 (observed) `[log] count is 0` printed while the app loaded, before the event loop started (it printed even in the
  no-display run). — verify: issue 7
- A16 (design, checked) Rationale for JS on the UI thread at event time: synchronous `preventDefault` and layout reads,
  atomic transactions. — `plans/research/skelter/README.md` Q6 (decided); code: `deliver.rs:1-18` delivers JS during the input's handling.
- A17 (design) Rust owns the shell plumbing (docking, tabs, toolbars, menus, windows); Svelte renders the contents of
  panels, dialogs, windows and overlays. — `CLAUDE.md` "Current focus"

## B. The module loader and the compiler

- B1 (code) `SourceConfig::for_path`: a directory app's entry is `src/App.svelte` (unless an `embed` table); app root is
  the nearest directory with `package.json`; `"svelters": {"toolkit"}` is the backend, default `gtk`.
  — `crates/svelters-modules/src/source/mod.rs:78-124`
- B2 (code) The generated entry module for an App: `import { mount } from '@svelters/native-runtime'; import App from …; mount(App, { props: {} });`
  — `mod.rs:376`
- B3 (code) Each `.svelte` file is compiled by `svelters_compiler::compile` inside the loader when it is imported;
  errors stop the load with the rendered diagnostics joined by newlines. — `crates/svelters-modules/src/source/transform.rs:85-118`
- B4 (code) Backends whose spec says `style.engine == "backend"` (egui, skelter) get the component's scoped `<style>` as
  text for their CSS engine. — `transform.rs:76-83`, `:94-100`; `metadata/semantics/skelter.json:10373-10375`
- B5 (code) Generated code imports `@svelters/native-runtime/core` as `$` (the reactive core) and
  `@svelters/native-runtime` as `$n` (the native runtime). — `crates/svelters-compiler/src/target.rs:10-21`
- B6 (code) The skelter semantic spec `metadata/semantics/skelter.json` is "Generated from skelter's kind registry
  (crates/svelters-skelter/src/spec.rs)" and compiled into the compiler. — `crates/svelters-compiler/src/semantics/backend.rs:14-15`, `:62`
- B7 (code) skelter.json `capabilities`: `ids`, `templates`, `native_transitions`, `native_motion`, `css_precompiled`.
  — `metadata/semantics/skelter.json:11-17`
- B8 (code) `Label.text` in skelter.json: `"change": "layout"`, `"id": 135398928`, type string. — `skelter.json:2920-2940` (id at `:2926`)
- B9 (code) `Button.clicked`: `"sync": false`, `"cancelable": false`, doc "Pressed and released". — `skelter.json:860-877` (id `171666320` at `:867`)
- B10 (code) `svelters-compile` is a binary of package `svelters-compiler`; it accepts `--backend skelter` (built-in spec list
  includes skelter); prints the generated JS on stdout and each diagnostic with `d.render(&filename)` on stderr; exit 1 on errors.
  — `crates/svelters-compiler/Cargo.toml:12-14`; `src/bin/svelters-compile.rs:1-8`, `:201-210`; `semantics/backend.rs:54-63`
- B11 (code) The CLI's rendered diagnostics use the same `Diagnostic::render` as the loader (`transform.rs:115`), with the
  file name as typed on the command line (`svelters-compile.rs:122`, `:202`). — inferred: same text as the loader's, untruncated.
- B12 (code; **not active for skelter**, see J4) With the `ids` capability, element creation is `$n.element_id($$K_Kind)` and a scalar prop write is
  `$n.setp(n, $$P_Kind_prop, …)`, with `const $$P_… = <id>;` constants and one `$n.ids(backend, hash, [...])` check per module;
  a toplevel keeps `$n.toplevel("window", …)`. — `crates/svelters-compiler/src/codegen/component.rs:869-877`, `:893-904`, `:929-948`;
  test `crates/svelters-compiler/tests/skelter.rs:43-58`
- B13 (code) With `templates`, a static subtree becomes a module-scope `$n.template(...)` table and one `$n.instantiate(...)`;
  nodes are fetched with `$n.at(instance, k)`; listeners of a template come in one `$n.template_listen(...)`; a dynamic
  label text is still a write inside the template effect. — test `tests/skelter.rs:72-93`
- B14 (code) A fragment has a single `$.template_effect` with every reactive write of its elements, "an unrelated change
  re-runs them all (each runtime write skips an unchanged value)"; call expressions are memoized: `$.template_effect(($0) => .., [() => f(x)])`
  "runs `f(x)` in a derived of its own, so it is not called again when only another value of the fragment changes".
  — `codegen/component.rs:710-714`, `:2063-2068`
- B15 (code) `$effect` compiles to `$.user_effect`. — `crates/svelters-compiler/src/transform/visit.rs:873`
- B16 (code) Error codes `unknown_element`, `text_not_allowed`, `unknown_property`, `unknown_event`. — `crates/svelters-compiler/src/codes.rs:73`, `:75`, `:80`, `:84` (round 1 F28)
- B17 (observed, round 1; **superseded by J2** for the draft's file: column 3:6) `<push-button>` from a short path: `Load: Error loading module '/tmp/hs/app/src/Bad.svelte': /tmp/hs/app/src/Bad.svelte:2:4:
  error[unknown_element] (skelter): <push-button> is not an element of the skelter backend` and
  `help: did you mean one of `button`, `radio-button`, `toggle-button`?`; exit 1. — verify: issue 2 (round 1 F73)
- B18 (observed) The loader's message is cut at about 256 bytes; with a long path only the path shows. — verify: issue 2
- B19 (observed) `onfoo` on `<button>` → `error[unknown_event] (skelter): <button> has no event `foo` on skelter`; `onclick`
  is also accepted on `<button>`. — verify: Step 4
- B20 (code) The loader and host have hot-reload pieces (`hmr` compile mode, a `reload` module); the runner never sets
  `cfg.hmr` and watches no files. — `mod.rs:65-66`, `:119`; `crates/svelters-host/src/lib.rs:40`; round 1 F20

## C. The native runtime (compiled code → host ops)

- C1 (code) "this module only turns component code into host ops ... There is no JS tree of DOM-like nodes and no anchors:
  containers and regions keep just the ordering native insertion needs." — `packages/native-runtime/src/index.js:1-6`
- C2 (code) Reactivity is stock Svelte 5.57.2's signals, effects and batching, through one adapter
  (`core/svelte.js` is the only importer of `svelte/internal/*`). — `index.js:2-4`; `packages/native-runtime/src/core/svelte.js:1-9`; `packages/native-runtime/package.json` (`"svelte": "5.57.2"`)
- C3 (code) `element(kind)` → `create` calls `host.create(kind)` and keeps a small JS `Native` record (kind, handle, last
  written values). — `index.js:527-545`, class `Native` at `:96-140`
- C4 (code) `set(node, prop, value)` returns early when the value equals the last one written (`last`), else calls
  `host.setProp`. — `index.js:672-716`
- C5 (code; not reached on skelter today, J4) `setp` (the ids fast path) dedupes in JS unless the host dedupes, then `hs.setp`/`hs.setProp`. — `index.js:718-729`
- C6 (code) `on(node, signal, fn)` registers the handler; the first handler for a signal makes the host `listen`. — `index.js:1006-1029`
- C7 (code) `__svelters_dispatch(h, signal, args, …)` finds the node by handle and runs its handlers. — `index.js:1229-1243`, `:1283-1305`
- C8 (code) `mount` installs `__svelters_dispatch`, `__svelters_dispatch_sync`, `__svelters_gone`, `__svelters_node`. — `index.js:3022-3028`
- C9 (code) A toplevel is shown after the outermost build returns: `presentTop` calls the spec's present method
  (`present` for a window). — `index.js:445-480`
- C10 (code) Templates: a backend that builds templates itself gets one `host.instantiate`; "Every other table, and every other
  host, expands into the ops the compiler writes for elements without templates, in the same order". — `packages/native-runtime/src/template.js:16-22`, `:38-45`
- C11 (code) `host.templates` is the backend's `has_templates()`; `Backend::supports_templates` defaults to `false` and
  `SkelterBackend` does not override it (its `impl Backend` at `backend.rs:834-1204` has no such method), so on skelter the
  runtime expands templates into one create/set/insert call per node. — `crates/svelters-host/src/quickjs.rs:1534`;
  `crates/svelters-abi/src/safe.rs:1672-1676`; grep: no `supports_templates` override in the workspace.

## D. The tree (skelter-core) and kinds

- D1 (code) "A Document holds every node of one app, across its windows, as columns indexed by node slot ... Each node has a
  Kind: a static KindSpec that declares its props with their ChangeClass, its signals and slots, and a value that measures,
  records, hit-tests, handles input and projects accessibility." — `crates/skelter-core/src/lib.rs:1-9`
- D2 (code) The `Kind` trait: `spec`, `prop_changed`, `layout_style`, `measure`, `prepare`, `record` (required), `hit_test`,
  `event`, `call`, `text_input`, `native_state`, `virtual_children`, `a11y`, … — `crates/skelter-core/src/kind.rs:485-560`
- D3 (code) Change classes: `Property` (compositor only), `Paint` (re-record this node), `Layout` (style, then layout if
  the intrinsic size changed, then record), `Structure` (relayout to the boundary, record, a11y, hit list).
  — `crates/skelter-base/src/change.rs:4-15`
- D4 (code) `Document::set_prop` → `write_prop`: coerces the type, returns at once when the stored value is unchanged
  (`if old == v`), lets the kind see it, stamps a revision, marks the work of the prop's class (`mark_class`), journals
  `Change::Prop`. — `crates/skelter-core/src/document.rs:795-797`, `:809-840`, `:858`, `:964`, `:969-991`
- D5 (code) A JS write to a `native_owned` prop that the kind changed after the event JS saw is refused as
  `Superseded`; the backend treats that as success ("the native value stands"). — `document.rs:820-832`; `backend.rs:663-670`
- D6 (code) Label `text` is a `Layout`-class prop; Button `text` is `Layout`. — `crates/skelter-widgets/src/controls/label.rs:43`; `controls/button.rs:72`
- D7 (code) `Document::frame` runs the UI phases (style, layout, prepare, record, property trees, layers, hit list, a11y) and
  returns the `Commit` delta, or `None` when nothing changed; the dirty walk "visits only nodes with record work".
  — `crates/skelter-core/src/frame.rs:1-15`, `:61-67`
- D8 (code) `Commit` carries "Re-recorded display lists only"; "Building it clones only what changed; the compositor folds
  commits". — `crates/skelter-base/src/commit.rs:16-29`
- D9 (code) Damage is computed by diffing flattened items against the previous frame's. — `crates/skelter-compositor/src/damage.rs:1-11`
- D10 (code) The base layout of every node is a column flexbox; CSS `flex-grow` is read into taffy. — `crates/skelter-core/src/style/cascade.rs:513-516`, `:535-537`
- D11 (code) Kinds JS can create: skelter-widgets' Tier 0 controls (box, button, label, checkbox, slider, …) and some views;
  "every other concrete kind gets a core kind synthesized from its surface spec ([`Placeholder`]: the props, signals and slots
  are real tree members, the node draws nothing) until its widget lands"; `window` is among those ("not yet `window`").
  — `crates/svelters-skelter/src/kinds.rs:4-9`, `:166-181`, `:538-549`; `crates/skelter-widgets/src/host.rs:512-538` (no window)
- D12 (code) Surface tags map to core tags (`combo-box` → `select`, `action` → `menu-item`), others unchanged. — `kinds.rs:141-146`
- D13 (code) Wire ids are skelter-core's FNV-1a hashes of (declaring kind, member, type). — `kinds.rs:18-22`; `skelter.json:20-23`
- D14 (code) `KindEntry` has `spec: &'static KindSpec` (with `name`), `text_prop: Option<PropId>`, and `prop(PropId) -> Option<&PropSpec>`
  (with `change`). — `crates/skelter-core/src/registry.rs:12-33`; `kind.rs:174-182`; `kind.rs` `PropSpec` at `:18-32`
- D15 (code) `Document` reads: `kind_of`, `children`, `get_prop`, `registry`. — `document.rs:268`, `:363`, `:369`, `:994`
- D16 (code) `SkelterBackend` public reads: `windows()`, `node(h)`, `handle(n)`, `with_doc(f)`, `traffic()`. — `backend.rs:451`, `:433`, `:438`, `:390`, `:242`
- D17 (code) `svl_handle` is `u64`. — `crates/svelters-abi/src/lib.rs:10`
- D18 (code) Handles count up from 1 and are never reused. — `backend.rs:73-91`
- D19 (code) Box "paints its own CSS box (background, border, shadows, outline)"; its layout props are the CSS properties
  of the same names; "a prop the runtime set wins over every rule". — `crates/skelter-widgets/src/controls/box_.rs:1-8`

## E. Input: how a click travels

- E1 (code) skelter-core dispatch: "for each platform event, the hit test on the last frame's hit list, the native defaults
  (hover, press, click, focus, …, the kinds' own handlers, innermost first), then the JS-facing events with their
  capture/target/bubble paths, which svelters-skelter delivers before the microtask drain." — `crates/skelter-core/src/input/dispatch.rs:1-4`
- E2 (code) `Document::dispatch` runs in one transaction; `hit_test` uses "the last frame's hit list". — `dispatch.rs:233-265`, `:267-270`
- E3 (code) Native handlers bubble from the target outwards until one handles the event. — `dispatch.rs:46-56`
- E4 (code) "Events with no listener on their path are not listed." — `crates/skelter-core/src/input/mod.rs:46`
- E5 (code) Button: Hoverable, Focusable, Pressable; on release it emits `press`/`release`/`clicked` signals; hover/press
  looks come from UA rules on state bits (`:hover` → `button-bg-hover`). — `crates/skelter-widgets/src/controls/button.rs:1-8`, `:58-60`, `:255-278`
- E6 (code) `SkelterBackend::dispatch` runs the input through skelter-core and delivers what it produced now; a non-sync
  signal is `post_event`ed; a closing `checkpoint` delivers and drains microtasks. — `crates/svelters-skelter/src/deliver.rs:1-18`, `:73-81`, `:155-203`
- E7 (code) Host checkpoint order: settle offloads, changed props batch, queued events (draining jobs after each), due
  timers, remaining jobs; "Ops reach the backend directly while JS runs (no batching in P)". — `crates/svelters-host/src/core.rs:1063-1079`
- E8 (code) The common DOM-style `click` signal bubbles, is cancelable and sync. — `crates/skelter-core/src/common.rs:189`
- E9 (code) `SkelterDocument::dispatch`: "native defaults, then JS at event time". — `app.rs:76-81`

## F. Console, errors, counters

- F1 (code) `HostConfig::default()`: `echo_console`, `echo_errors` on; `turn_budget` 10 s; memory limit 256 MiB.
  — `crates/svelters-host/src/core.rs:95-114`
- F2 (code) Console lines go to stderr as `[<level>] <text>`; JS errors reported by the host as `[svelters] <err>`.
  — `core.rs:591-597`, `:617-630`; `crates/svelters-host/src/prelude.js:4-22`
- F3 (observed) `[log] count is 0/1/2` on stderr after two clicks. — verify: Step 4
- F4 (code) `Traffic` counters (Debug, pub fields): native→JS `syncs`, `posts`, `changes`, `checkpoints`, `frames`;
  JS→native `creates`, `tree`, `sets`, `gets`, `calls`, `listens`, `styles`. — `backend.rs:139-173`
- F5 (code) `Backend::create` counts `creates`; `set_prop` counts `sets`; `get_prop` counts `gets`; `call` counts
  `calls`; `listen_ex` counts `listens`. — `backend.rs:857-866`, `:903-909`, `:911-912`, `:935-936`, `:1085-1086`
- F6 (code) `Embed::logic` runs after every platform callback (`SveltersApp::sync`) and counts a checkpoint each time.
  — `embed/mod.rs:633-643`; `app.rs:173-177`
- F7 (code) `window_desc` reads three props through `get_prop` (counted as `gets`). — `backend.rs:763-774`
- F8 (code) The compositor logs with the `log` crate, e.g. `log::info!("skelter-compositor: {} on {:?} ({:?})", info.name,
  info.backend, info.device_type)` when it creates the device, and `log::error!` for adapter/device/surface failures.
  — `crates/skelter-compositor/src/gpu.rs:217-265`, `:286-290`; `crates/skelter-compositor/Cargo.toml:13` (`log = "0.4"`)
- F9 (code) `log` 0.4.34 is already in `Cargo.lock`; no logger implementation (env_logger) is. — `Cargo.lock:3039-3041`
- F10 (observed) The runner installs no logger; with GL on llvmpipe under Xvfb the window stayed black and nothing was printed.
  — verify: issue 5

## G. Style, theme, the black window

- G1 (observed) The first window: 360×200, titled "Hello skelter", black background, light grey text flush top-left
  (the window spec's default padding 8 not visible). — verify: Step 3 issue 6 (round 1 F69)
- G2 (code) `install_widget_services` installs `skelter_widgets::theme::Theme::default()`, which is "Dark Modern".
  — `app.rs:35-48`; `crates/skelter-widgets/src/theme/mod.rs:538-543`
- G3 (code) `Theme::light_modern()` exists; `skelter_widgets::host::install(doc, text, policy, theme)` is public; `Native::for_os(...).policy()`. — `theme/mod.rs:556`; `host.rs:428`; `interact/policy.rs:350`, `:358`
- G4 (code) The compositor clears damaged areas to `Color::TRANSPARENT` by default ("Clear colour of damaged areas").
  — `crates/skelter-compositor/src/lib.rs:84-85`, `:103`
- G5 (inferred from G4, D11, G2; **confirmed by J10**) The black is the cleared, transparent window surface showing through: the `window` kind is a
  placeholder that records nothing, and only kinds that paint (box with a CSS background, button) draw rectangles. The light text is
  the Dark Modern theme's foreground. Platform windows are opaque: `transparent: false` — `crates/skelter-platform/src/api.rs:217`.
- G6 (code) The window's `padding` default (8) is a surface spec default of a placeholder kind; layout comes from the CSS cascade,
  which starts from a plain column flexbox (`cascade.rs:513-516`). — inferred: explains G1's flush text.
- G7 (code) The `style` op: sheets, classes, id, inline style go to the tree, which runs svelters-style's cascade. — `crates/svelters-skelter/src/style.rs:1-5`
- G8 (observed) `fixtures/counter-egui` (with a `<style>` block, `.column`/`.row` gaps) runs on skelter from source and its gaps appear.
  — verify: "Where to go next"; `fixtures/counter-egui/src/App.svelte:24-33`

## H. Extension points (where Svelte ends and Rust begins)

- H1 (code) An app registers its own core kinds with `Kinds::with_app`; they are creatable from JS with their spec's props,
  methods and signals and published into the generated spec. — `crates/svelters-skelter/src/kinds.rs:575-582`; `tests/app_kinds.rs:1-5`
- H2 (code) `use:` behaviours: Rust code attached to an element from Svelte (`use:pick={params}` with
  `import { pick } from 'svelters:behaviors/cad'`); not a tree node; `register_behavior(ns, name, factory)`; `add_event_tap`
  sees each dispatched input. — `crates/svelters-skelter/src/behavior.rs:1-16`, `:115`, `:145`
- H3 (code) `set_state_names`: the host's provider states for `:state(ns.name)` in the app's sheets. — `backend.rs:563-569`
- H4 (code) `SkelterBackend::shared`: a backend over the host's own document, "JS's nodes are nodes of the host's tree".
  — `backend.rs:303-317`
- H5 (code, cadatomic) cadatomic shares its shell window's document with the backend (`SkelterBackend::shared`), registers its kinds
  with `Kinds::with_app`, the behaviours `command`, `op`, `live`, `presents` under `cad`, an event tap, and `:state(cad.*)` names
  (`cad.pending`, `cad.stale`, `cad.failed`, …). — cadatomic: `crates/cadatomic-app/src/skelter/tree.rs:1-10`, `:67`;
  `crates/cadatomic-ui/src/skelter/embed.rs:21`, `:24-33`, `:66-73`; `crates/cadatomic-ui/src/skelter/mod.rs:56-64`
- H6 (design, checked) "Direct manipulation is native and never waits on JS: hover, press, focus, caret, scroll, drags,
  transitions." — `plans/research/skelter/README.md` summary 3; code: E5 (button hover/press from state bits), D5 (native-owned props).
- H7 (design, checked) Why retained: egui "repaints the whole window and rebuilds every visible widget each frame"; mature toolkits
  keep a retained tree re-recorded only where dirty. — `plans/research/skelter/README.md` summary 1; code: D7, D8.
- H8 (design) The svelters decision: "One retained scene, owned by skelter-core; cadatomic extends it through kind, behaviour and
  state providers." — `CLAUDE.md` decisions list.

## I. Setup (round 1, verified)

- I1 (observed) `pnpm install`: ~5 s, 366 packages, boxed warning about esbuild's ignored build script. — verify: Step 1
- I2 (observed) First build: 388 crates, 2 min 44 s on 4 cores, `CARGO_PROFILE_DEV_DEBUG=0`, 2.4 GB. — verify: issue 11
- I3 (observed) X11 needs libxkbcommon-x11 (panic otherwise); no Vulkan driver → `svelters-skelter: cannot open window #1:
  Failed to create surface for any enabled backend: {}`, exit 0; no display → `… event loop: … neither WAYLAND_DISPLAY nor
  WAYLAND_SOCKET nor DISPLAY is set.`, exit 1; missing pnpm install → `… is not installed (pnpm install)`, exit 1.
  — verify: Step 3 issues 1, 4; Step 3 "Missing pnpm install"
- I4 (observed) The Step 5 crate (`examples/hello-skelter`, workspace member, `main.rs` as in round 1) built and ran; same window,
  same log lines, exit 0 on close; Cargo.lock gained the package. — verify: Step 5
- I5 (code) Rust 1.97.0 pinned; Node ≥ 22, pnpm 10.33.0. — `rust-toolchain.toml:2`; `package.json:6-8`
- I6 (code) The workspace has `skelter-widgets` and `skelter-core` path deps. — `Cargo.toml:58`, `:66`

## J. Round 2 run of draft0 (verify.md) and draft1 additions

All observed in svelters `6c1ea458`, Ubuntu 24.04, Xvfb, Mesa 25.2.8 lavapipe, rustc 1.97.0 (verify.md header).

- J1 (observed) `pnpm install` 1.9 s, `Packages: +366`, esbuild warning; cold build of runner + CLI 2 min 07 s on 4 cores
  (round 1: 2 min 44 s). Draft1 says "a few seconds", "between two and three minutes". — verify.md execution log, "Setting up"
- J2 (observed) `<push-button>` at line 3 of the draft's App.svelte → column `3:6` from both the loader and the CLI; short path
  `/tmp/hs2/app` gives the full message with help line; from the deep worktree path only `Load: Error loading module '<path>': <path…`
  (262 bytes, cut). `onfoo` on `<label>` → `…App.svelte:3:12: error[unknown_event] (skelter): <label> has no event `foo` on skelter`.
  — verify.md execution log, "Experiment: break it on purpose"
- J3 (code) The cut: QuickJS formats every thrown error into `char buf[256]` (`rquickjs-sys-0.14.0/quickjs/quickjs.c:8431-8434`);
  "Error loading module '…'" is rquickjs's wording (`rquickjs-core-0.14.0/src/result.rs:453-461`); `Load:` is svelters-host's
  `ErrorKind::Load`. With a `/home/you/svelters` checkout the message is ~310 bytes, so the cut falls in the help line (arithmetic).
  — verify.md, issues 2 and 11
- J4 (observed + code) The compiled counter (`runs/App.js`, 28 lines) writes props by name: `$n.set($$n3, "text", …)`; it has no
  `$$P_…` constants, no `$n.ids`, no `$n.setp`. It does have `$n.toplevel("window", "present")`, `$n.instantiate($$T0, $n.slot($$n1, null), [() => 8])`,
  `$n.at`, `$n.append`, `$.template_effect`, `$n.template_listen($$t0, [() => $.update(count)])`, `$.user_effect`, and the table
  `$$T0 = $n.template("299f0516aa46760e", [["box", null, [["setv", "gap", 0]], [1, 2, 3], null], …], [[3, "clicked", 0]])`.
  Cause: skelter.json nests `spec_hash` under `"ids"` (`metadata/semantics/skelter.json:20-23`; `crates/svelters-skelter/src/spec.rs:400`)
  while the compiler reads a top-level `spec_hash` (`crates/svelters-compiler/src/semantics/spec.rs:62`, `compile.rs:261-264`), so the ids
  mode stays off. Draft1 says only "built, listed, doesn't switch on for skelter in current builds".
  — verify.md execution log, "Experiment: read what the compiler made"; explanations table row "Compiler emits `$$P_Label_text`"
- J5 (code) The host takes the prop name and resolves it per call: `setProp(h, name, value)` → `conv.with_name(&name, |n| c.op_set_prop(…))`
  — `crates/svelters-host/src/quickjs.rs:1176-1188`; runtime `set` dedupes against `last` then calls `node.host.setProp(node.h, prop, value)`
  — `packages/native-runtime/src/index.js:672-716`. `$n.ids` "verifies them against the host once per module" — `crates/svelters-compiler/src/codegen/component.rs:928-929` (doc comment).
- J6 (observed) First `svelters-compile` run builds the CLI in 23.8 s even with the runner built. — verify.md execution log,
  "Experiment: read what the compiler made" (`runs/compile.stderr`)
- J7 (observed) Logging variant: startup prints `template reads count 0`, `template reads doubled 0`, `effect: count is 0`; one click
  prints the five lines in draft1's block. — verify.md execution log, "Experiment: watch the order of events" (`runs/s3-order.stderr`)
- J8 (observed) With `name` + Rename button, one Increment then one Rename: startup adds `template reads name Ada` before the effect line;
  Increment logs no name line; Rename logs only `template reads name Grace`. Compiled thunk with Rename: `() => show('name', $.get(name))`;
  without any assignment: `() => show('name', name)`. — verify.md execution log, "Experiment: watch the order of events"; issue 9
  (`runs/s11-name.stderr`, `runs/app-name2/src/App.svelte`)
- J9 (observed) Tree dump exactly as draft1 shows, preceded by `[log] count is 0`. Traffic: untouched `syncs 0, posts 0, changes 0,
  checkpoints 8, frames 0, creates 5, tree 4, sets 7, gets 3, calls 1, listens 1, styles 7`; two clicks `posts 2, checkpoints 27, sets 11`;
  five moves `checkpoints 117`, rest unchanged. Plain `{name}` label: `creates 6, tree 5, sets 8` at startup, `sets 12` after two clicks.
  `<style>` block: `styles 8`. — verify.md execution log, "Experiment: look at the tree, and count the crossings", "Why the window is black"
  (`runs/s5-t*.stderr`, `runs/s9-name*.stderr`, `runs/s6-panel.stderr`)
- J10 (observed) Panel rule: grey panel full width, content height, 12 px inset, black below; `flex-grow: 1` fills. Light theme: dark text,
  light button, black background. `c.clear = Some(skelter_base::color::Color::from_rgba8(0, 0, 128, 255))` → whole background navy, button and
  text unchanged (run with the light theme installed). — verify.md execution log, "Why the window is black"; `shots/s6-*.png`, `s7-light-0.png`,
  `s10-clear-0.png`; API: `crates/skelter-compositor/src/lib.rs:84-85` (`clear: Option<Color>`, "`None` = draw over the previous frame"),
  `crates/skelter-base/src/color.rs:14`, `:21`
- J11 (observed) Hover: fill `gray(49)` over and off the button in every move-only run, with timer-driven frames running and with a
  pointer-move listener that was called; the hover fill `gray(59)` appears only after a click and stays after the pointer leaves.
  Draft1 calls it a probable repaint bug. — verify.md execution log, "Experiment: look at the tree…" (last bullet); issue 3
- J12 (code) Change classes with examples: Property "transform, opacity, clip, scroll offset…"; Paint "colours, borders, icons…"; Layout
  "text, font, padding, width, gap"; Structure "create, insert, remove, reorder, slot". — `crates/skelter-base/src/change.rs:6-15`.
  skelter.json counts: property 8, paint 127, layout 193, structure 0 (grep).
- J13 (code) Traffic field docs: `syncs` "synchronous calls (`call_sync`: cancelable events…)"; `posts` "events posted for the next
  checkpoint"; `changes` "observed native prop changes reported"; `checkpoints`/`frames` "checkpoints run … and animation frames";
  `styles` style ops. — `crates/svelters-skelter/src/backend.rs:139-173`. `changes` counted where `host.prop_changed` is called
  — `crates/svelters-skelter/src/deliver.rs:141-151`. `Button.clicked` `"sync": false` — `skelter.json:860-877`.
- J14 (observed) No display: stderr `[log] count is 0` first, then `svelters-skelter-run: event loop: os error at …/winit-0.30.13/src/platform_impl/linux/mod.rs:765:
  neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.`, exit 1. `ldd`: vdso, libgcc_s, libm, libc, ld-linux. Missing
  node_modules: `…package "svelte" of alias target "svelte/src/easing/index.js" is not installed (pnpm install)`, exit 1.
  — verify.md execution log, "When something goes wrong"
- J15 (observed) Logger run with one click: two winit `[WARN]` lines (XSETTINGS, XRandR 0mm), two `[ERROR] wgpu_hal::gles::egl … DRI2: failed
  to load driver`, then `[INFO] skelter_compositor::gpu: skelter-compositor: llvmpipe (LLVM 20.1.2, 256 bits) on Vulkan (Cpu)`; traffic
  `posts 1, checkpoints 21, sets 9`. — verify.md execution log, "When something goes wrong" (`runs/s8-log.stderr`). "The WARNs come from the
  bare virtual X server" is inferred (Xvfb, no settings daemon).
- J16 (observed) `VK_ICD_FILENAMES=/nonexistent.json` with the logger: window black with no text; Vulkan loader errors, EGL errors, then
  `[ERROR] skelter_compositor::gpu: skelter-compositor: no GPU adapter for backends [Backends(VULKAN), Backends(GL)]`; exit 0 on close. Through
  the runner: only `[log] count is 0`. — verify.md execution log, "When something goes wrong" (`runs/s8-gl*.stderr`); issue 6
- J17 (observed, round 1) Without libxkbcommon-x11: `thread 'main' panicked at …/xkbcommon-dl-0.4.2/src/x11.rs:59:28: Library libxkbcommon-x11.so could not be loaded.`;
  with no Vulkan driver: `svelters-skelter: cannot open window #1: Failed to create surface for any enabled backend: {}`, exit 0.
  Not re-run in round 2. — `exp/sv-tutorial-first-skelter-app/verify.md` issues 1 and 4
- J18 (code) `install_widget_services` picks the platform with `cfg!(target_os)`; draft1's light-theme snippet hard-codes `Platform::Linux`.
  — `crates/svelters-skelter/src/app.rs:38-45`; verify.md issue 13
- J19 (code) `$n.append(container, node)` records the node in the container's items and attaches it. — `packages/native-runtime/src/index.js:1478-1482`
- J20 Not run anywhere: Wayland, a real GPU, a machine where GL works. Draft1 says so where it matters.

## Code blocks in draft0 and their sources

- K1 `package.json`, K2 first `App.svelte`, K3 run command: round 1 C1, C2, C4 (verified).
- K4 counter `App.svelte`: round 1 C3 (verified); K4b the logging variant adds `show()` and a handler function — not run (P2).
- K5 `svelters-compile` command: B10. Not run.
- K6 `Cargo.toml`, K7 members edit, K8 `main.rs`: round 1 C5–C7 (verified).
- K9 the dump/traffic additions: written from D14–D17, F4; compiled and ran (J9).
- K10 `<style>` panel: B4, D19, D10, G7; not run (P4).
- K11 light theme swap: G3; compiled and ran (J10).
- K12 stderr logger: F8, F9; `log` 0.4 API (`set_logger`, `set_max_level`, `Log` trait); compiled and ran (J15).
- K13 (draft1) clear-colour `spawn` replacement: J10. K14 (draft1) Rename button: J8. K15 (draft1) plain `{name}` traffic label: J9.

## Predictions draft0 asked the reader to check (all run in round 2; outcomes in J)

Draft1 replaces each with the observed output: P1 → J4 (ids part wrong), P2 → J7/J8, P3 → J9 (and J11 for the hover look),
P4 → J10, P5 → J10, P6 → J15.

- P1 The compiler's output for the counter contains `$n.toplevel("window"`, a `$n.template(` table, `$n.instantiate(`,
  a `$.template_effect(`, `$n.template_listen(` (or `$n.on(`) with `$.update(count)`, `$.user_effect(`, and
  `const $$P_Label_text = 135398928;`. From B12–B15, B8.
- P2 Per click, stderr order: handler lines, then the template's `show(...)` calls for the labels whose value changed,
  then the `$effect` line; a label reading only `name` logs nothing on click. From B14, C2 (stock Svelte flush: render
  effects before user effects, both in the microtask drain after the handler), E7.
- P3 `Traffic` per click: `posts` +1 (one `clicked` delivery), `sets` +2 (two label texts); hovering adds neither; an
  unchanged `{name}` label adds no `sets`. From E4–E6, C4/C5, F5. `checkpoints` grows with every callback (F6), so ignore it.
- P4 A `.panel { background-color; padding }` box draws a filled rectangle as tall as its content and as wide as the window;
  with `flex-grow: 1` it fills the window. From D19, D10. Not run.
- P5 With `Theme::light_modern()` the label text turns dark while the background stays black. From G3, G5. Not run.
- P6 With the logger, one line names the GPU adapter and backend, e.g. `… skelter-compositor: llvmpipe (…) on Vulkan (Cpu)`. From F8.
