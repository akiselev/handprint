# sv-howto-skelter: verifier report (stage 2)

Checked against svelters `6c1ea458` and cadatomic `b350c24` (the shas the fact sheet names; both
checkouts are at them). Paths: `sv/` = /home/user/svelters, `cad/` = /home/user/cadatomic. Nothing
was built; every verdict comes from reading the source.

## Claim verdicts (facts.md)

| # | Verdict | Note |
|---|---|---|
| F1 | ok | Minor: `spec()` is also required (sv/crates/skelter-core/src/kind.rs:487), so `record` (:511) is the only required *instance* hook. |
| F2 | ok | kind.rs:498-526. |
| F3 | ok | kind.rs:174-230. |
| F4 | ok | value.rs:35-46. Note that the type part is the prop's type, the **signal's argument type** and the **method's return type** (`void` when there is none); see kind.rs:234-246 and draft issue 5. |
| F5 | ok | registry.rs:111-116. |
| F6 | ok | registry.rs:55-75. `MemberCollision` = within one kind; `CrossKindCollision` = against another kind (or the common members). |
| F7 | ok | tests/app_kinds.rs:12-53. |
| F8 | ok | app_kinds.rs:22-33. |
| F9 | ok | cx.rs:577-600; testkinds.rs:128-129. |
| F10 | doubtful (incomplete) | The signatures are right (cx.rs:568, :296), but `draw_text` returns `None` and draws nothing without a document text shaper (cx.rs:566-570, `self.shaper.as_deref_mut()?`). Neither the runner nor `install_widget_services` calls `Document::set_text_shaper` (document.rs:264). Only tests and cadatomic do (cad/crates/cadatomic-app/src/skelter/mod.rs:81). `measure_text` uses the `TextMeasure` given to `with_text`, so it does work. |
| F11 | ok | cx.rs:89-146, 281-292, 466-491. |
| F12 | ok | cx.rs:712-714; document.rs:1263. |
| F13 | ok | kinds.rs:575-615 (`with_app` at :582; `register_hosted` registers skelter-widgets' kinds, :166-174). |
| F14 | ok | kinds.rs:522-535; tier0.rs `WIDGET` :75. |
| F15 | ok | kinds.rs:458-505. |
| F16 | ok | kinds.rs:267, app_kinds.rs:115. |
| F17 | ok | surface/mod.rs:18-20; kinds.rs:566-568. |
| F18 | ok | cad/.../skelter/embed.rs:26-34. |
| F19 | ok | kinds.rs:522 (also `kind_name` leaks, :446-456). |
| F20 | ok | kinds.rs:131-137; app_kinds.rs:119. |
| F21 | ok | cad/.../skelter/mod.rs:66-75 (also `forwarded::<>` and the `property_grid`/`code_editor` registrars). |
| F22 | ok | status_text.rs:11-40. |
| F23 | ok | quantity_field.rs:10-11. |
| F24 | ok | backend.rs:292-301. |
| F25 | ok | backend.rs:311-328. `shared` takes `Rc<RefCell<Document>>`, not a `Document`. |
| F26 | ok | svelters-skelter-run.rs:48-73, :16. |
| F27 | ok | app.rs:35-48. It sets services only, not the document's text shaper (see F10). |
| F28 | ok | sv/Cargo.toml:54; cad/Cargo.toml:59-65; cadatomic-ui/Cargo.toml:45. |
| F29 | ok | spec.rs:417-447. |
| F30 | ok | spec.rs:199-204. **But `fragment` writes only `app` kinds** (spec.rs:425), so a fragment never contains `"placeholder"` (draft issue 2). |
| F31 | ok | spec.rs:130-146. |
| F32 | ok | app_kinds.rs:172-181. |
| F33 | ok | codes.rs:73, 80, 82, 84. |
| F34 | ok | svelters-compile.rs:34, 77-87. |
| F35 | ok | source/mod.rs:49-62, 108-118. |
| F36 | ok | semantics/spec.rs:804-812. The kind id here is the kind *name* (`QuantityField`), not the tag; a duplicate tag gives a separate error, "spec fragment: tag <x> is already defined" (:822). |
| F37 | ok | embed.rs:36-63; source.rs:116-131. Note that cadatomic **assigns** `c.spec_fragments = vec![…]` and so drops the package.json entry on purpose (draft issue 1). |
| F38 | ok | app_values.rs:51-81 (assigns as well, :80). |
| F39 | ok | app_kinds.rs:173, 141 (the :141 enum write goes through the ABI, not compiled Svelte). |
| F40 | ok | app_values.rs:110-139. |
| F41 | ok | app_values.rs:89-108. |
| F42 | ok | The cadatomic comment at quantity_field.rs:68-72 is backed by the runtime: sv/packages/native-runtime/src/index.js:1180-1183, 1297, 1301. |
| F43 | ok | StatusPrompt.svelte:5. |
| F44 | ok | Gallery.svelte:59-74. The `--sv-*` variables resolve there because cadatomic adds its own theme sheet (cad/crates/cadatomic-ui/src/theme/skelter.rs:153,182), not because svelters-skelter does (draft issue 3). |
| F45 | ok | behavior.rs:1-12. |
| F46 | ok | behavior.rs:26-46. |
| F47 | ok | behavior.rs:115. |
| F48 | ok | behavior.rs:121-139; tests/behaviors.rs:83-90. |
| F49 | ok | source/mod.rs:390-397. |
| F50 | ok | Confirmed by grep: the only writers of `.svelters/app/behaviors` are svelters-egui's dump (svelters-egui/src/spec.rs:1181-1259), and the vite alias only reads that path (packages/native-runtime/vite.js:176). |
| F51 | ok | behavior.rs:141-147; tests/behaviors.rs:21-37, 108-123. |
| F52 | ok | embed.rs:21-22, 65-74; Gallery.svelte:9, 25-26. |
| F53 | ok | svelters-style/src/state.rs:1-7. |
| F54 | ok | state.rs:143-153. |
| F55 | ok | backend.rs:564-569. |
| F56 | ok (message) / doubtful (when it surfaces) | selector.rs:235, 252. In the dev path (text sheets) the diagnostic is thrown away and the rule silently never matches (skelter-core/src/style/mod.rs:97-105; svelters-style/src/sheet.rs:102-113). The message reaches anyone only for precompiled sheets (svelters-skelter/src/style.rs:31; svelters-style/src/binary.rs:282). |
| F57 | ok | mod.rs:38-64 (the bits live in `pub mod cad`); embed.rs:69. |
| F58 | ok | app.rs:7-13; examples/egui-kinds/app/src/App.svelte:19. That example is an **egui** app; there is no skelter example app in `examples/`. |
| F59 | ok | svelters-compile.rs:34, 101-105; semantics/backend.rs:62. |
| F60 | ok | tier0.rs (Widget `height`/`width`; Button `text`, `clicked`); JobsPanel.svelte:27. |
| Note (`class`) | ok | svelters-skelter/src/style.rs:83-95. |

Tally: 59 ok, 1 doubtful (F10), 0 wrong, 0 unsupported. F56 is ok on the text of the message but doubtful
on when anyone sees it, and it is counted as ok. The fact sheet is accurate. The draft's problems come
from what it puts together from these facts and from what it leaves out.

## Draft issues

1. **"`cfg.spec_fragments.push(fragment);` … This is the pattern the svelters-skelter tests and cadatomic both use" and the package.json block `"specFragments": ["app.skelter.json"]`.**
   Verdict: wrong, and it breaks the build. `for_path` already loads `specFragments` from package.json
   (source/mod.rs:113-117). `push` then adds the same file a second time, and `spec_backend` merges
   every entry with no de-duplication (source/mod.rs:352). The second merge fails with "spec fragment:
   kind QuantityField is already defined" (semantics/spec.rs:810-812). A reader who follows both
   step 3 and the package.json example hits this. Neither cited source pushes: the test assigns
   (`cfg.spec_fragments = vec![dir.join("app.skelter.json")]`, tests/app_values.rs:80), and cadatomic
   assigns to drop the egui dump that package.json names (cad/crates/cadatomic-ui/src/svelte/source.rs:120-131).
   Fix: use `cfg.spec_fragments = vec![fragment];` in steps 3 and 4. Present package.json
   `specFragments` as the alternative for a fragment written ahead of time (for vite or
   `svelters-compile`), and say not to do both.

2. **"If the fragment says `"widget": "placeholder"` for your tag, your `register` closure never ran for it."**
   Verdict: wrong. `spec::fragment` writes only kinds whose `m.app` is true (svelters-skelter/src/spec.rs:425),
   so a placeholder never appears in the fragment. When `register` did not run for a tag, that tag is
   simply missing from the fragment's `"tags"`, and the compiler then reports `unknown_element`.
   Fix: "If your tag is missing from the fragment's `tags`, your `register` closure did not register it."
   Move that sentence to the `unknown_element` item.

3. **"A window shows a rounded box beside a `+1` button"**, with the style `background-color: var(--sv-list-hover);`.
   Verdict: doubtful, probably wrong. On skelter nothing defines `--sv-*` for author CSS. The token sheet
   (`default_tokens_css`, sv/crates/skelter-widgets/src/theme/mod.rs:771) is installed only by
   svelters-egui (sv/crates/svelters-egui/src/style.rs:811). `install_widget_services` puts the `Theme`
   in a service for skelter-widgets' painters (app.rs:47; skelter-widgets/src/host.rs:428-434) and adds
   no sheet. cadatomic adds its own theme sheet (cad/crates/cadatomic-ui/src/theme/skelter.rs:153,182).
   So in the reader's app the `var()` is undefined, the background computes to transparent, and the
   box is invisible. Fix: use a literal colour and a border (`background-color: #dde3ea; border: 1px solid #889;`),
   or say that the `--sv-*` variables exist only once the app adds `Theme::css()` as a sheet.

4. **"To draw text, use `cx.draw_text(text, at, max_width)` in `record` … Both use the node's computed font."**
   Verdict: incomplete; the reader sees no text. `RecordCx::draw_text` shapes with the document's
   `TextShaper` and returns `None` when there is none (skelter-core/src/cx.rs:566-570). The step 4
   runner and `install_widget_services` never call `Document::set_text_shaper` (document.rs:264). In
   the tree, only tests and cadatomic set one (cad/crates/cadatomic-app/src/skelter/mod.rs:81).
   `skelter_text::TextSystem` implements `skelter_core::TextShaper` (sv/crates/skelter-text/src/system.rs:676).
   Fix: add
   `backend.with_doc(|d| d.set_text_shaper(Box::new(skelter_text::TextSystem::new(fonts.clone()))));`
   to step 4 and mark it as not taken from the stock runner. Also say that `draw_text` returns `None`
   when no shaper is set.

5. **"Each member's id is a hash of the kind's tag, the member's name and its type name (`number` for `F64`, `string` for `Str`, and so on)"**
   Verdict: partly wrong. For a signal the type part is its argument type, and for a method it is the
   **return** type, `"void"` when there is none (kind.rs:239-246; tests/app_values.rs:38 uses
   `MethodId::of("probe", "fail", "void")`). A reader who writes a method with no return value and
   copies the pattern gets an id that never matches. Fix: "…its type name: a prop's type, a signal's
   argument type, a method's return type (`void` when it has none). `step` returns a number, so its id
   uses `"number"`."

6. **The `Run` behaviour block, introduced with "This is from svelters-skelter's tests", and step 2's `move || Box::new(Run(c.clone()))`.**
   Verdict: does not compile as shown. `Calls` is the test's own alias, `Rc<RefCell<Vec<String>>>`
   (tests/behaviors.rs:19), and `c` is a clone made in the test's `rig()` (:41-43). Neither is defined
   on the page, and the `use` lines are missing: `svelters_skelter::behavior::{Behavior, BehaviorCx, Props}`,
   `skelter_core::{ListenFlags, common as core_common}` (:13, :17). Fix: add the imports and the
   `type Calls` and `let c = calls.clone();` lines, or replace the call log with a small struct that
   holds the command string.

7. **"Register it on the backend" / "Write it before you load the app", with `backend.behavior_modules()` and `cfg.root`.**
   Verdict: unstated placement that the reader will get wrong. In step 4's `main`, `backend` is moved
   into `Embed::with_backend(backend, …)` and `cfg` is moved into `SourceProvider::new(cfg)`. The
   behaviour calls have to come before both moves, or run on a clone (`SkelterBackend` is `Clone`;
   tests/app_values.rs:76 uses `backend.clone()`). Fix: say "in `main`, after `install_widget_services`
   and before `Embed::with_backend`". Write the modules after `cfg` is built and before
   `SourceProvider::new`.

8. **"A svelters checkout … depends on them by path … You also need `skelter-core`, `skelter-base`, … and `svelters-host`."**
   Verdict: incomplete. The page never shows a `[dependencies]` block. It also leaves out two manifest
   lines that cadatomic copies from svelters because they do not reach dependents:
   - the AccessKit fork under `[patch.crates-io]` (sv/Cargo.toml:125-128; cad/Cargo.toml:115-121,
     "All three crates from the one rev, as svelters' root manifest has them");
   - `[profile.dev.package.rquickjs-sys] opt-level = 2` (cad/Cargo.toml:134-138, "Profiles do not
     propagate from dependencies"; without it JS runs 5-10x slower and can hit the handler budget).

   The skelter-widgets route also needs `skelter-widgets`. Fix: give a complete `[dependencies]` block
   with every path from the list, plus the patch and profile lines. Mark the patch as copied from
   svelters' root manifest at the checkout's sha.

9. **`SourceConfig::for_path(Path::new("ui"))?` and "write your own `main`".**
   Verdict: unstated file locations. The page never says that `main` goes in a binary crate of the
   reader's own, which Cargo.toml it changes, or that `"ui"` is a directory relative to the working
   directory that must hold `package.json` and, for a directory, `src/App.svelte`
   (source/mod.rs:85-101). Fix: one sentence each, for example "`ui/` is your Svelte app: `ui/package.json`
   and `ui/src/App.svelte`, the component below."

10. **"for example with `svelters-skelter-run --entry <app dir>`" and "`svelters-compile --backend skelter --spec-fragment app.skelter.json App.svelte`".**
    Verdict: the reader cannot run these as written. Both are binaries inside the svelters workspace,
    not installed tools (crates/svelters-skelter/src/bin/svelters-skelter-run.rs; crates/svelters-compiler/src/bin/svelters-compile.rs).
    `svelters-compile`'s usage text lists only `gtk|qt|appkit|winui|egui` (:34), but `skelter` works
    (semantics/backend.rs:62). There is also no skelter example app to start from: `examples/` holds
    egui, gtk, qt, mac and win apps only. Fix: `cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry <dir>`
    and `cargo run -p svelters-compiler --bin svelters-compile -- --backend skelter …`; add "(the usage
    text does not list `skelter` yet, but it is accepted)". Consider `--check`, which reports
    diagnostics without printing JS. Say plainly that svelters has no skelter starter app: the reader's
    app needs a package.json with `"svelters": { "toolkit": "skelter" }`, a `src/App.svelte` with a
    `<window>`, and `@svelters/native-runtime` installed (pnpm), or a svelters checkout to fall back
    on (source/mod.rs:134-143).

11. **"`spec fragment: kind ... is already defined`. A tag in your fragment is already in the built-in spec. Pick a tag that no built-in kind uses."**
    Verdict: incomplete or misleading. The id is the derived kind *name* (`quantity-field` → `QuantityField`,
    kinds.rs:446-456), not the tag. The usual cause is merging the same fragment twice (issue 1).
    Registering a built-in tag that skelter-widgets leaves as a placeholder also triggers it, because
    the app then owns that tag and the fragment re-emits the built-in spec (kinds.rs:590-594, 595-600).
    A duplicate tag has its own message, `spec fragment: tag <x> is already defined` (semantics/spec.rs:824).
    Fix: list both causes and both messages.

12. **"`... hash to the same id ...: rename one`: two member ids collide, so rename one of the members."**
    Verdict: slightly wrong. That message is `CrossKindCollision`: one of your members hashes to the
    same id as a member of another registered kind, or as your kind's id (registry.rs:62, 72-73, 141-148).
    A collision inside one kind reads "kind `k`: members `a` and `b` have the same id" (registry.rs:71).
    Fix: name both messages, and say that the cross-kind one means renaming your member (or the tag).

13. **"If your host already owns a `skelter_core::Document`, use `SkelterBackend::shared(kinds, doc)` instead. That document's registry must contain every kind in `kinds` under the same tag, which `svelters_skelter::kinds::register_builtin` adds."**
    Verdict: partly wrong. `register_builtin` adds only the built-ins (kinds.rs:148-161). The app's kinds
    must be registered into the host's registry too; the troubleshooting item says so, but this
    sentence contradicts it. `shared` takes `Rc<RefCell<Document>>` (backend.rs:317). Fix: "…which you
    build with `register_builtin` plus your own `register`. Pass the document as `Rc<RefCell<Document>>`."

14. **"Steps 2 to 5 are the same for both."**
    Verdict: slightly wrong. With skelter-widgets' interface, step 2's closure calls
    `skelter_widgets::host::register::<K>(reg)` instead of `reg.register::<K>()` (cad/.../skelter/mod.rs:68-69),
    and the crate needs `skelter-widgets`. Fix: "Steps 3 to 5 are the same; in step 2, register it with
    `skelter_widgets::host::register::<K>(reg)?`."

15. **"`:state()` troubleshooting: `unknown provider state `x` in :state()`. The sheet was parsed before `set_state_names`, or the name isn't in your table."**
    Verdict: doubtful. Dev builds send `<style>` as text. `Stylesheet::parse` records the error as a
    diagnostic that skelter-core discards (svelters-style/src/sheet.rs:107-113; skelter-core/src/style/mod.rs:97-105),
    so the rule silently never matches and no message appears. The message surfaces only for
    precompiled release sheets, prefixed "precompiled sheet: selector `…`:" (svelters-style/src/binary.rs:282;
    svelters-skelter/src/style.rs:31). Fix: make the symptom "a `:state()` rule never matches". In a
    release build the sheet fails to load with that message. Give the same two causes.

16. **"This kind is adapted from svelters-skelter's test of app kinds. The only change is the `cx.draw_box()` call"**
    Verdict: minor inaccuracy. The `doc` string also changed (the test's is "A number with a unit
    (stand-in for cadatomic's).", app_kinds.rs:31), and the `use` line drops `KindRegistry` and
    `SlotSpec`. The code is fine. Fix: "Apart from the doc string, the only change is…".

17. **The package.json block `{ "svelters": { "toolkit": "skelter", "specFragments": ["app.skelter.json"] } }`.**
    Verdict: not marked illustrative. facts.md calls it illustrative ("Block 5"), and it conflicts with
    issue 1 as written. Fix: mark it as an example and keep it consistent with the corrected step 3.

18. **"The component compiles with no diagnostics."**
    Verdict: unsupported. The tests assert only that there are no errors, or no `unknown_*`
    diagnostics (app_kinds.rs:176-181; app_values.rs:85). Nothing shows that this assembled component
    is warning-free; for example, Svelte may warn about the non-`$state` `let field;`. Fix: "compiles
    without errors".

19. **"Pass `svelters_skelter::surface::builtin()` (the tier 0 and tier 1 built-ins)".**
    Verdict: internal jargon. "Tier 0/1" is svelters' internal grouping and means nothing to a reader.
    Fix: "(svelters-skelter's built-in kinds, including the `Widget` base…)".

20. **cadatomic is named throughout ("as cadatomic does", "what cadatomic uses for `<quantity-field>`") and is never introduced.**
    Verdict: an outside reader won't know what it is. cadatomic's real `<quantity-field>` is also a
    different kind from the page's: a string `value`, `commit`/`submit`/`edit` signals, and no
    `changed` (cad/.../skelter/quantity_field.rs:53-82). A reader who compares the two will be
    confused. Fix: introduce it once ("cadatomic, a CAD application built on svelters, registers its
    kinds this way") and say that the page's `<quantity-field>` is the simplified stand-in from
    svelters' tests.

21. **The troubleshooting headings `**`kind `x` collides with registered kind `y``**` and `**`unknown provider state `x` in :state()`.**`.**
    Verdict: broken Markdown. The backticks inside the code spans end them early. Fix: use
    double-backtick spans (`` `kind `x` collides…` ``) or plain quotes.

22. **The heading "## Steps".**
    Verdict: a template-word heading (brief: headings name the subject). Fix: for example "## Add the
    `<quantity-field>` kind".

## Writer's doubts

- (a) **Confirmed.** Nothing in svelters-skelter writes `.svelters/app/behaviors/<ns>.js`;
  `behavior_modules()` only returns the text (behavior.rs:121-139). The only writer is svelters-egui's
  dump (svelters-egui/src/spec.rs:1251-1259), and cadatomic's `cad.js` was produced by that egui dump
  (cad/ui/.svelters/app/behaviors/cad.js:1). The draft's statement and the illustrative writer block
  are right. The block's position in `main` needs stating (issue 7).
- (b) **`--backend skelter` works** (semantics/backend.rs:62); only the usage text is stale
  (svelters-compile.rs:1, 34). The draft's command is right. Telling the reader about the stale usage
  text and how to run the binary would help (issue 10).
- (c) **Assembled component:** every element and attribute exists. `<window title default-width default-height>`,
  `<box flex-direction gap>`, `<button text onclicked>` and `<label text>` are in tier0.rs, and
  `width`/`height`/`class` are Widget props or the style op. `onchanged={(v) => …}` gets the value
  because the signal is posted (index.js:1301). `el.call` exists (index.js:161), and JS `1` reaches
  `as_f64` (value.rs:83-88). What it will *look* like is wrong (issue 3).
  **Runner `main`:** by inspection it compiles, given step 2's `app_kinds` and the dependencies. Every
  call matches the stock runner, with `?`/`map_err` in place of `fail`. It is missing the text shaper
  (issue 4), and the `push` breaks when package.json lists the fragment too (issue 1).
  **Behaviour-module writer:** correct and marked illustrative; only its placement is unstated (issue 7).

## Counts

- Fact-sheet claims: 60 (+1 note) checked: 59 ok, 1 doubtful (F10), 0 wrong, 0 unsupported.
- Draft issues: 22. By verdict: 4 wrong, or broken as written (1, 2, 5, 6); 3 doubtful or likely
  wrong behaviour (3, 4, 15); 5 incomplete or missing steps a reader needs (7, 8, 9, 10, 13);
  10 minor (11, 12, 14, 16-22).
- Most serious: 1 (the fragment merged twice fails the compile), 3 (the theme variable is undefined
  on skelter, so the box is invisible), 4 (`draw_text` draws nothing without a document shaper).
