# sv-howto-skelter: fact sheet

## Topic choice

**Chosen: (b) "Add a native widget kind and use it from Svelte".**

Why: the whole path is built and tested end to end, and has a real consumer.
- `Kinds::with_app` (svelters-skelter/src/kinds.rs:575-615) registers app kinds; `tests/app_kinds.rs` covers
  registration, creation through the backend, a method call emitting a signal, the spec fragment, and the
  compiler accepting the tags after the merge. `tests/app_values.rs` goes through real svelters-host with compiled
  Svelte (record props, `el.call`, coded errors). `tests/behaviors.rs` covers `use:` behaviours.
- cadatomic uses exactly this API in production code (`cadatomic-ui/src/skelter/embed.rs`, `mod.rs`).
- The CSS topic (a) is also built, but it spans svelters-style's large engine, the compiler's sheet path and the
  theme JSON; "what is unsupported" would need a much wider audit to state without guessing. (b) can be grounded
  claim by claim from a few files.

Shas read: svelters `6c1ea458` (2026-10-26 19:45 UTC), cadatomic `b350c24` (2026-10-26 20:43 UTC).
Paths below: `sv/` = /home/user/svelters, `cad/` = /home/user/cadatomic.

## Kinds and registration

- F1. A skelter-core kind is a type implementing `skelter_core::Kind`; `spec()` returns a `&'static KindSpec` and
  `record` is the only required hook. sv/crates/skelter-core/src/kind.rs:484-511
- F2. Default hooks: `measure` returns `taffy::Size::ZERO`; `layout_style` is `cx.default_layout()`; `event` returns
  `Handled::No`; `call` returns `PropError::Rejected("{method:?} is not implemented by this kind")`.
  sv/crates/skelter-core/src/kind.rs:493-526
- F3. `KindSpec` fields: `name` (the tag, matched by CSS type selectors), `props`, `signals`, `slots`, `role`,
  `text_prop`, `focusable`, `container`, `methods`, `name_from`, `parts`, `doc` and more; `KindSpec::new(name)` gives
  defaults (GenericContainer role, not focusable, no members). sv/crates/skelter-core/src/kind.rs:174-230
- F4. Member ids: `PropId::of(kind, member, type-name)`, where type names are `bool`, `int`, `number` (F64),
  `string`, `enum`, `color`, `list`, `record`. sv/crates/skelter-core/src/kind.rs:234-245;
  sv/crates/skelter-core/src/value.rs:35-46
- F5. `KindRegistry::register::<K: Kind + Default>()` registers through `register_with(K::spec(), factory)`;
  `register_with` takes a spec and a factory. sv/crates/skelter-core/src/registry.rs:111-116
- F6. Registry errors: `DuplicateKind` ("kind `{name}` collides with registered kind `{existing}`"),
  `MemberCollision`, `CrossKindCollision` ("... hash to the same id ...: rename one"), `UnknownTextProp`.
  sv/crates/skelter-core/src/registry.rs:55-75, 116-152
- F7. Example kind with prop, signal, method, `call` that uses `cx.view(cx.node())...prop_f64`, `cx.set_native`,
  `cx.emit` (copied in the draft). sv/crates/svelters-skelter/tests/app_kinds.rs:12-53
- F8. `PropSpec::new(name, ValueType, ChangeClass).default(..).native().doc(..)`; `SignalSpec::new(name, Some(type),
  SignalFlags::NONE).args(&[ArgSpec::new(..)])`; `MethodSpec::new(name, &[args], Some(ret))`.
  sv/crates/svelters-skelter/tests/app_kinds.rs:22-33
- F9. `RecordCx::draw_box()` draws the CSS box (shadows, background, border, outline) from the computed style.
  sv/crates/skelter-core/src/cx.rs:577-600. `BoxKind::record` calls `cx.draw_box()` first.
  sv/crates/skelter-core/src/testkinds.rs:128-135
- F10. `RecordCx::draw_text(text, at, max_width)` shapes with the document's `TextShaper` and draws in the node's
  font and colour; it returns `None` and draws nothing without a shaper (`self.shaper.as_deref_mut()?`).
  sv/crates/skelter-core/src/cx.rs:566-575. `MeasureCx::measure_text` uses the `TextMeasure` given to `with_text`.
  cx.rs:294-298. (Corrected in stage 3: the shaper requirement was missing.)
- F11. `RecordCx` and `MeasureCx` deref to `NodeView`, which has `prop`, `prop_f64`, `prop_str`, `prop_bool`, `text`,
  `style`, `bounds`. sv/crates/skelter-core/src/cx.rs:89-146, 281-292, 466-491
- F12. `EventCx::set_state(bits, on)` sets state bits on the node; `Document::set_state` restyles when they change.
  sv/crates/skelter-core/src/cx.rs:712-714; sv/crates/skelter-core/src/document.rs:1262-1266

## svelters-skelter: Kinds::with_app

- F13. `Kinds::with_app(surface, app)` registers skelter-widgets' kinds, then calls `app(&mut reg)`, then synthesizes
  placeholders for remaining surface kinds; an app kind's JS-facing spec comes from its core `KindSpec`
  (`surface_of`) unless `surface` already has a spec with its tag. `surface` must hold `surface::tier0::WIDGET`.
  sv/crates/svelters-skelter/src/kinds.rs:575-615
- F14. App kinds' surface specs extend `surface::tier0::WIDGET`, so they inherit the Widget props (e.g. `anchor`,
  `flex-grow`). sv/crates/svelters-skelter/src/kinds.rs:522-535; sv/crates/svelters-skelter/src/surface/tier0.rs:13,29,75-80
- F15. In the JS-facing spec, a prop with an `Unset` default is nullable ("an unset default reads as null"); a signal
  with no named args but an arg type gets one arg named `value`; containers get a default `children` slot before
  named ones. sv/crates/svelters-skelter/src/kinds.rs:458-505; tests/app_kinds.rs:117,121
- F16. The app kind's own members keep their skelter-core ids on the wire (masked with `WIRE_MASK`).
  sv/crates/svelters-skelter/tests/app_kinds.rs:115; src/kinds.rs:263-276
- F17. `surface::builtin()` = tier0 + tier1 kinds. sv/crates/svelters-skelter/src/surface/mod.rs:18-20.
  `Kinds::builtin()` uses it. src/kinds.rs:566-568
- F18. cadatomic builds its kinds with `Kinds::with_app(&svelters_skelter::surface::builtin(), super::register)` once
  per thread in a `thread_local!`. cad/crates/cadatomic-ui/src/skelter/embed.rs:26-34
- F19. `surface_of` leaks the derived spec with `Box::leak` (build the registry once).
  sv/crates/svelters-skelter/src/kinds.rs:522
- F20. Inherited `Widget` props map to skelter-core's common props: `enabled`↔`disabled` (inverted), `visible`↔
  `hidden`, `accessible-name`→`aria-label`, `tooltip`→`title`, `autofocus`. src/kinds.rs:131-137; tests/app_kinds.rs:119
- F21. cadatomic's `register` registers its kinds, both through skelter-widgets' host
  (`skelter_widgets::host::register`) and directly (`reg.register::<quantity_field::QuantityField>()`).
  cad/crates/cadatomic-ui/src/skelter/mod.rs:66-75
- F22. skelter-widgets' kind interface: `kind_props!` macro, `skelter_widgets::shim::core::{Kind, KindSpec, ...}`,
  painted with `skelter_widgets::paint`. cad/crates/cadatomic-ui/src/skelter/status_text.rs:11-113
- F23. cadatomic's `quantity-field` is a skelter-core kind directly because a text field needs `Event::Ime` and
  `text_input`, "which skelter-widgets' kind interface does not carry". cad/.../quantity_field.rs:10-11

## Backend and runner

- F24. `SkelterBackend::new(kinds)` (no text measurement) and `SkelterBackend::with_text(kinds, text)`.
  sv/crates/svelters-skelter/src/backend.rs:292-301
- F25. `SkelterBackend::shared(kinds, doc: Rc<RefCell<Document>>)` checks the host document's registry holds every kind; error text "the host
  document's registry lacks <{tag}>: build it on svelters_skelter::builtin_kinds().registry(), or register them into it
  with svelters_skelter::kinds::register_builtin". backend.rs:311-328
- F26. The stock runner `svelters-skelter-run` uses `svelters_skelter::builtin_kinds()`, `install_widget_services`,
  `Embed::with_backend(backend, EmbedConfig::app(host))`, a `SourceConfig::for_path` with `backend = "skelter"`,
  `embed.load_app_from`, `skelter::compositor(...)`, `skelter_platform::run(Box::new(SveltersApp::new(embed, spawn)))`.
  sv/crates/svelters-skelter/src/bin/svelters-skelter-run.rs:48-73. Usage line :16.
- F27. `install_widget_services` gives the document skelter-widgets' text *service*, the OS policy and the default
  theme, through `skelter_widgets::host::install` (`set_service` calls only). It does not set the document's text
  shaper and adds no style sheet. sv/crates/svelters-skelter/src/app.rs:35-48; sv/crates/skelter-widgets/src/host.rs:428-434
- F28. Crates are `publish = false` (workspace); cadatomic depends on them by path to `../svelters/crates/...`.
  sv/Cargo.toml:54; cad/Cargo.toml:62; cad/crates/cadatomic-ui/Cargo.toml:45

## Spec fragment and compiler

- F29. `spec::fragment(&kinds)` / `fragment_text(&kinds)`: the app kinds only (`kinds` and `tags`), merged into
  the built-in `skelter.json` by `svelters-compile --spec-fragment kinds.json` or `SourceConfig::spec_fragments`; "a
  tag or kind already there is an error". sv/crates/svelters-skelter/src/spec.rs:417-447
- F30. In the generated spec, an app kind has `"widget": "app"`; a placeholder has `"placeholder"`; skelter-widgets'
  `"skelter-widgets"`. spec.rs:199-204. The *fragment* holds only app kinds (`m.app`), so it never contains a
  placeholder. spec.rs:425
- F31. A native prop (`.native()`) gets a two-way binding whose event is the kind's `changed` signal (else the first
  signal with an argument named like the prop). spec.rs:130-146; tests/app_kinds.rs:164
- F32. Without the fragment the compiler reports `unknown_*` diagnostics for the app tags; with it, none.
  tests/app_kinds.rs:172-181
- F33. `unknown_element`: "The tag is not an element of the backend's semantic spec (nor a component in scope)."
  sv/crates/svelters-compiler/src/codes.rs:73; also `unknown_property` :80, `unknown_method` :82, `unknown_event` :84.
- F34. `svelters-compile --explain <code | id>` exists. sv/crates/svelters-compiler/src/bin/svelters-compile.rs:34
- F35. `SourceConfig` has `spec_fragments: Vec<PathBuf>`; `for_path` reads it from package.json
  `"svelters": { "specFragments": [...] }` relative to the app root. sv/crates/svelters-modules/src/source/mod.rs:49-62, 108-118
- F36. Merge errors: "spec fragment for backend {b:?} cannot extend the {} spec", "spec fragment: kind {id} is already
  defined" (`id` is the kind *name*, e.g. `QuantityField`, derived from the tag by `kind_name`,
  sv/crates/svelters-skelter/src/kinds.rs:446-456), and "spec fragment: tag <{tag}> is already defined".
  sv/crates/svelters-compiler/src/semantics/spec.rs:804-825
- F37. cadatomic writes `svelters_skelter::spec::fragment(&kinds())` to a temp `kinds.json` per process and sets
  `c.backend = "skelter"`, `c.spec_fragments = vec![fragment_path]`; its comment: egui's dumped `kinds.json` "cannot
  extend skelter's spec". cad/crates/cadatomic-ui/src/skelter/embed.rs:36-63; cad/crates/cadatomic-ui/src/svelte/source.rs:116-131
- F38. The test scratch app writes `fragment_text(kinds)` to `app.skelter.json` and sets
  `cfg.spec_fragments = vec![dir.join("app.skelter.json")]`. tests/app_values.rs:51-81

## Svelte side

- F39. Svelte usage compiled in the test: `<property-grid><quantity-field unit="mm" onchanged={() => {}} /></property-grid>`,
  `<status-text tone="error" />`. tests/app_kinds.rs:173. Enum props set by name string: tests/app_kinds.rs:141
- F40. Methods are called from Svelte with `bind:this={el}` and `el.call('echo', ...)`; a `PropError::coded("E_RANGE",
  ...)` arrives as a JS `Error` with `.code`. tests/app_values.rs:110-139
- F41. A record prop set from Svelte arrives as a record with nested fields. tests/app_values.rs:89-108
- F42. A signal with `SignalFlags::NONE` is posted; its listeners get the arguments alone; a sync signal's listeners get
  the event object first. cad/crates/cadatomic-ui/src/skelter/quantity_field.rs:68-72
- F43. Real cadatomic Svelte: `<status-text anchor="prompt" section="prompt" flex-grow={1} />`.
  cad/ui/panels/status/StatusPrompt.svelte:4-6
- F44. Real CSS on classes, `:state(cad.*)` and theme variables: `.presenter:state(cad.pick-target) { background-color:
  var(--sv-list-hover); outline: 1px solid var(--sv-focus); }`. cad/ui/panels/gallery/Gallery.svelte:59-74. The
  variables resolve there only because cadatomic adds its own theme sheet (F64).

## Behaviours (`use:`)

- F45. `use:name={params}` with `import { name } from 'svelters:behaviors/<ns>'`; the action creates a node of kind
  `<ns>.<name>`, sets params, inserts, updates, destroys. On skelter the behaviour is no tree node: the backend routes
  insert → `Behavior::attach(cx, props)`, a prop write → `update` with the whole record, remove/destroy/element freed →
  `detach`. Params are untyped; `null` removes one. sv/crates/svelters-skelter/src/behavior.rs:1-12
- F46. `trait Behavior { fn attach(&mut self, cx, props); fn update(..) {} ; fn detach(..) {} }`, `BehaviorCx { handle,
  node, doc }`, `Props = BTreeMap<String, Value>`. behavior.rs:26-46
- F47. `SkelterBackend::register_behavior(ns, name, factory)`. behavior.rs:113-119
- F48. `behavior_modules()` returns `(ns, source)` with `export const {name} = nativeBehavior('{tag}', []);`, to be
  written at `<root>/.svelters/app/behaviors/<ns>.js`. behavior.rs:121-139; tests/behaviors.rs:83-90
- F49. The module loader resolves `svelters:behaviors/<ns>` to `<root>/.svelters/app/behaviors/<ns>.js`; namespace
  must be lowercase ASCII, digits, `_` or `-`; a missing file errors with "(run the app binary's dump)".
  sv/crates/svelters-modules/src/source/mod.rs:390-397
- F50. Nothing in svelters-skelter writes that file itself (grep: only `behavior_modules` returns the text; cadatomic's
  `ui/.svelters/app/behaviors/cad.js` header says it is generated by svelters-egui's spec.rs). cad/ui/.svelters/app/behaviors/cad.js:1
- F51. `add_event_tap(|doc, ev, events| ..)` sees every dispatched input and the JS-facing events it produced, before
  JS. behavior.rs:141-147. A behaviour listens natively with `cx.doc.listen(cx.node, core_common::CLICK,
  ListenFlags::PASSIVE)` so the tap sees the click. tests/behaviors.rs:21-37, 108-123
- F52. cadatomic registers `command`, `op`, `live`, `presents` under `cad` and adds a tap.
  cad/crates/cadatomic-ui/src/skelter/embed.rs:21-22, 65-74; Svelte use `use:presents={{ type: 'mcad.face', key:
  'face-1', owner: 'gallery-body' }}`. cad/ui/panels/gallery/Gallery.svelte:9,25-26

## State providers (`:state()`)

- F53. State bits 0–31 are svelters' built-ins; 32–63 are provider states. sv/crates/svelters-style/src/state.rs:1-7
- F54. `StateNames` trait (`lookup(name) -> Option<StateMask>`), implemented for `[(&str, StateMask); N]`. state.rs:143-153
- F55. `SkelterBackend::set_state_names(names)`: "sheets sent from now on resolve through `names`". backend.rs:564-569
- F56. An unknown name in `:state()` is a parse error: "unknown provider state `{n}` in :state()". sv/crates/svelters-style/src/selector.rs:235,252.
  For text sheets (dev) skelter-core drops the diagnostic, so the rule silently never matches
  (sv/crates/skelter-core/src/style/mod.rs:97-105); only a precompiled sheet reports it
  (sv/crates/svelters-style/src/binary.rs:282; sv/crates/svelters-skelter/src/style.rs:31).
- F57. cadatomic's provider bits: `pub const PENDING: StateMask = StateMask(1 << 32);` etc., `STATES` table, and
  `backend.set_state_names(Box::new(super::STATES))` in `install`. cad/crates/cadatomic-ui/src/skelter/mod.rs:38-64; embed.rs:68-69

## Code block sources (draft0)

- Block 1 (kind): sv/crates/svelters-skelter/tests/app_kinds.rs:12-53, with `cx.draw_box()` added in `record`
  (from sv/crates/skelter-core/src/testkinds.rs:129) — marked adapted.
- Block 2 (register): adapted from tests/app_kinds.rs:97-106 and cad/.../embed.rs:26-34.
- Block 3 (fragment + config): adapted from tests/app_values.rs:63,78-81 and cad/.../source.rs:126-131.
- Block 4 (runner): sv/crates/svelters-skelter/src/bin/svelters-skelter-run.rs:48-73, with `builtin_kinds()` replaced.
- Block 5 (package.json): illustrative, field names from F35.
- Block 6 (Svelte): adapted from tests/app_kinds.rs:173 and tests/app_values.rs:110-124; CSS pattern from Gallery.svelte.
- Block 7 (behaviour): sv/crates/svelters-skelter/tests/behaviors.rs:21-45.
- Block 8 (behaviour module write): illustrative, from F48/F49.
- Block 9 (state names): cad/crates/cadatomic-ui/src/skelter/mod.rs:42,56-64 and embed.rs:69.

## Added while checking the draft

- F58. The runner opens a platform window for each `<window>` JS presents, with its title and
  `default-width`/`default-height`. sv/crates/svelters-skelter/src/app.rs:7-13. Real `<window title="Kinds"
  default-width={360} default-height={240}>` root: sv/examples/egui-kinds/app/src/App.svelte:19
- F59. `svelters-compile` resolves `--backend` through `SpecBackend::builtin(first)`, which knows `skelter`
  (the usage string lists only gtk|qt|appkit|winui|egui). sv/crates/svelters-compiler/src/bin/svelters-compile.rs:34,101-105;
  sv/crates/svelters-compiler/src/semantics/backend.rs:62-63
- F60. Widget base has `width`/`height` (`F64`, null for auto). sv/crates/svelters-skelter/src/surface/tier0.rs:31,49.
  Button has `text` and the `clicked` signal; real `onclicked` use: cad/ui/panels/jobs/JobsPanel.svelte:27
- Note: `class` is not a Widget prop; classes go through the style op (`SVL_STYLE_CLASSES`).
  sv/crates/svelters-skelter/src/style.rs:83-95. Removed `class` from the draft's list of common props.

## Added in stage 3 (reviser), same shas

- F61. Member id type part: prop → its type; signal → its arg type or `"void"`; method → its return type or `"void"`.
  sv/crates/skelter-core/src/kind.rs:234-245
- F62. `SourceConfig::for_path` fills `spec_fragments` from package.json `specFragments`; `spec_backend` folds every
  entry with `merge_fragment_path`, no de-duplication, so listing one file twice fails the second merge.
  sv/crates/svelters-modules/src/source/mod.rs:113-117, 346-352. Tests and cadatomic assign the list:
  sv/crates/svelters-skelter/tests/app_values.rs:80; cad/crates/cadatomic-ui/src/svelte/source.rs:120-131
- F63. `Document::set_text_shaper(Box<dyn TextShaper>)` sv/crates/skelter-core/src/document.rs:263-266;
  `SkelterBackend::with_doc(f)` sv/crates/svelters-skelter/src/backend.rs:389-392; `skelter_text::TextSystem`
  implements `skelter_core::TextShaper` under skelter-text's default `core` feature
  (sv/crates/skelter-text/src/system.rs:674-681; sv/crates/skelter-text/Cargo.toml:20-23). cadatomic sets a shaper
  on its document: cad/crates/cadatomic-app/src/skelter/mod.rs:75-82 (wrapper `DocShaper`, host.rs:135). The stock
  runner does not: sv/crates/svelters-skelter/src/bin/svelters-skelter-run.rs:48-51
- F64. No `--sv-*` sheet on skelter: the token sheet `default_tokens_css` (sv/crates/skelter-widgets/src/theme/mod.rs:771)
  is installed only by svelters-egui (sv/crates/svelters-egui/src/style.rs:803-812); svelters-skelter's only
  `add_sheet` is the component style op (sv/crates/svelters-skelter/src/style.rs:56). `Theme::css()` is the `:root`
  token sheet (theme/mod.rs:763-766); `Document::add_sheet(&str)` (sv/crates/skelter-core/src/document.rs:1196).
  cadatomic adds `theme.css()` + its base CSS with `doc.add_sheet` (cad/crates/cadatomic-ui/src/theme/skelter.rs:151-153,174-182).
- F65. A kind's computed box defaults: background and border colour transparent, colour black, font
  `system-ui, sans-serif` 13px. sv/crates/skelter-core/src/style/computed.rs:107-125
- F66. skelter-core's `TextKind::record`: `cx.draw_box(); let w = cx.size().w; cx.draw_text(text, Point::ZERO, Some(w));`
  sv/crates/skelter-core/src/testkinds.rs:184-189. `RecordCx` derefs to `NodeView` (`prop_f64`, `prop_str`, `size`)
  sv/crates/skelter-core/src/cx.rs:486-492, 120-143
- F67. Runtime lookup: app `node_modules` → `@svelters/build`'s → the svelters checkout's `packages/native-runtime`
  (via `CARGO_MANIFEST_DIR`); "not installed (pnpm install)" otherwise. sv/crates/svelters-modules/src/source/mod.rs:134-143.
  Svelte is resolved through the runtime's alias table from the runtime's own packages
  (sv/crates/svelters-modules/src/source/aliases.rs:23-46; sv/packages/native-runtime/package.json `svelters.aliases`).
  cadatomic's ui installs nothing: cad/ui/package.json `description`.
- F68. Manifest lines that don't propagate: AccessKit `[patch.crates-io]` (sv/Cargo.toml:125-128; cad/Cargo.toml:115-121)
  and `[profile.dev.package.rquickjs-sys] opt-level = 2` (sv/Cargo.toml:114-115; cad/Cargo.toml:134-138).
- F69. `RegistryError::MemberCollision` text "kind `{kind}`: members `{a}` and `{b}` have the same id";
  `CrossKindCollision` "`{kind}.{member}` and `{other_kind}.{other}` hash to the same id {id}: rename one".
  sv/crates/skelter-core/src/registry.rs:68-73
- F70. `Kinds::registry()` sv/crates/svelters-skelter/src/kinds.rs:665-667; `register_builtin` adds only the built-ins
  (kinds.rs:148-161).
- F71. `svelters-compile --check` reports diagnostics without code (svelters-compile.rs:13, 74); binaries run as
  `cargo run -p svelters-compiler --bin svelters-compile` (crates/svelters-compiler/Cargo.toml:12-14) and
  `-p svelters-skelter --bin svelters-skelter-run` (src/bin/).
- F72. Behaviour test scaffolding: imports and `type Calls = Rc<RefCell<Vec<String>>>`, `let c = calls.clone();
  r.backend.register_behavior(...)`. sv/crates/svelters-skelter/tests/behaviors.rs:13-19, 39-45
- F73. Directory entry: `for_path(dir)` mounts `src/App.svelte` when package.json has no `embed`; the root is the
  nearest directory with a package.json. sv/crates/svelters-modules/src/source/mod.rs:85-107

## Code block sources (draft1 changes)

- Block 1: `record` now adapted from testkinds.rs:184-189 (F66); `QTY_UNIT` added (F61).
- Cargo.toml block: dependency list from F28/cad/Cargo.toml:58-65; patch and profile copied (F68).
- package.json: illustrative, adapted from tests/app_values.rs:63.
- Block 3/4: `cfg.spec_fragments = vec![fragment]` (F62); `set_text_shaper` line added (F63), marked as not in the runner.
- Svelte style: literal colours (F64, F65); token-sheet one-liner built from F64's APIs, not copied from a file.
- Behaviour blocks: imports and `Calls` from F72.
