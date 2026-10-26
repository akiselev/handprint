# Add a native widget kind to a skelter app and use it from Svelte

This guide adds a widget written in Rust to a svelters app that runs on skelter, and then uses it from a Svelte
component as an ordinary element: `<quantity-field unit="mm" onchanged={...} />`. When you finish, the element
compiles, draws its box and its value, its props and methods reach your Rust code, and its signals arrive in Svelte
as event callbacks.

The page also covers attaching Rust code to an existing element with `use:`, and your own `:state()`
pseudo-classes.

cadatomic, a CAD application built on svelters, registers its own kinds this way and is cited below. The
`<quantity-field>` built here is the simplified stand-in from svelters' tests, not cadatomic's real text field.

## Before you start

You need:

- A svelters checkout beside your project, with `pnpm install` run once in it. Its crates are not published, so
  your binary crate depends on them by path. Svelte and `@svelters/native-runtime` come from the checkout when your
  app has no `node_modules`, so nothing is installed in your app.
- Familiarity with skelter-core's `Kind` trait.

Your binary crate's `Cargo.toml` needs these lines. The last two tables are copied from svelters' root manifest at
your checkout's commit, because patches and profiles don't reach dependent crates.

```toml
[dependencies]
skelter = { path = "../svelters/crates/skelter" }
skelter-base = { path = "../svelters/crates/skelter-base" }
skelter-core = { path = "../svelters/crates/skelter-core" }
skelter-compositor = { path = "../svelters/crates/skelter-compositor" }
skelter-platform = { path = "../svelters/crates/skelter-platform" }
skelter-text = { path = "../svelters/crates/skelter-text" }
svelters-host = { path = "../svelters/crates/svelters-host" }
svelters-skelter = { path = "../svelters/crates/svelters-skelter" }

# QuickJS is C; at opt-level 0 it runs 5-10x slower.
[profile.dev.package.rquickjs-sys]
opt-level = 2

# The AccessKit fork svelters builds with; all three crates from the one rev.
[patch.crates-io]
accesskit = { git = "https://github.com/akiselev/accesskit", rev = "b22e1418f323f15c93ed0d76e16d4c097ba6cce0" }
accesskit_atspi_common = { git = "https://github.com/akiselev/accesskit", rev = "b22e1418f323f15c93ed0d76e16d4c097ba6cce0" }
accesskit_consumer = { git = "https://github.com/akiselev/accesskit", rev = "b22e1418f323f15c93ed0d76e16d4c097ba6cce0" }
```

The Svelte app lives in `ui/` next to the crate: `ui/package.json` and `ui/src/App.svelte` (step 5). svelters has
no skelter starter app to copy; a minimal `package.json` is enough (an example, adapted from svelters-skelter's
tests):

```json
{ "name": "my-app", "private": true, "type": "module", "svelters": { "toolkit": "skelter" } }
```

This guide uses skelter-core's `Kind` trait directly, as svelters-skelter's tests do. skelter-widgets has a kind
interface of its own (the `kind_props!` macro, `skelter_widgets::shim::core::Kind`, painting through
`skelter_widgets::paint`) that gives you its text painting and theme; cadatomic's `<status-text>`
(`crates/cadatomic-ui/src/skelter/status_text.rs`) is a short example. With it, add the `skelter-widgets` crate and
register the kind in step 2 with `skelter_widgets::host::register::<K>(reg)?`; the other steps are the same.

## Add the `<quantity-field>` kind

### 1. Write the kind

A kind is a type that implements `Kind` and describes itself with a static `KindSpec`, which lists the props,
signals and methods that Svelte can use. Each member's id is a hash of the kind's tag, the member's name and a type
name: a prop's type, a signal's argument type, or a method's return type (`void` when there is none), spelled
`number` for `F64`, `string` for `Str`, and so on. `step` returns a number, so its id uses `"number"`. The
constants you match on are built with `PropId::of`, `SignalId::of` and `MethodId::of`.

This kind is adapted from svelters-skelter's test of app kinds. The doc string is shorter, and `record` is new: it
draws the element's CSS box (background, border, outline) and then the value and unit as text, the way skelter-core's
own text kind does:

```rust
use skelter_base::accesskit::Role;
use skelter_base::change::ChangeClass;
use skelter_base::geom::Point;
use skelter_core::{self as core, ArgSpec, EventCx, Kind, MethodId, MethodSpec, PropDefault, PropId, PropSpec, RecordCx, SignalFlags, SignalId, SignalSpec, Value as CValue, ValueType};

const QTY_VALUE: PropId = PropId::of("quantity-field", "value", "number");
const QTY_UNIT: PropId = PropId::of("quantity-field", "unit", "string"); // added, for `record`
const QTY_CHANGED: SignalId = SignalId::of("quantity-field", "changed", "number");
const QTY_STEP: MethodId = MethodId::of("quantity-field", "step", "number");

static QUANTITY_FIELD: core::KindSpec = core::KindSpec {
    props: &[
        PropSpec::new("value", ValueType::F64, ChangeClass::Paint).default(PropDefault::F64(0.0)).native().doc("The quantity in its unit (`bind:value`)."),
        PropSpec::new("unit", ValueType::Str, ChangeClass::Layout).doc("The unit shown after the number (`mm`)."),
    ],
    signals: &[SignalSpec::new("changed", Some(ValueType::F64), SignalFlags::NONE).args(&[ArgSpec::new("value", ValueType::F64)]).doc("The user changed the value.")],
    methods: &[MethodSpec::new("step", &[ArgSpec::new("by", ValueType::F64)], Some(ArgSpec::new("value", ValueType::F64))).doc("Adds `by`; returns the new value.")],
    role: Role::SpinButton,
    focusable: true,
    doc: "A number with a unit.",
    ..core::KindSpec::new("quantity-field")
};

#[derive(Default)]
struct QuantityField;

impl Kind for QuantityField {
    fn spec() -> &'static core::KindSpec {
        &QUANTITY_FIELD
    }
    fn record(&self, cx: &mut RecordCx) {
        // added: the CSS box, then the text (as skelter-core's `TextKind::record`)
        cx.draw_box();
        let text = format!("{} {}", cx.prop_f64(QTY_VALUE).unwrap_or(0.0), cx.prop_str(QTY_UNIT).unwrap_or(""));
        let w = cx.size().w;
        cx.draw_text(&text, Point::ZERO, Some(w));
    }
    fn call(&mut self, cx: &mut EventCx, method: MethodId, args: &[CValue]) -> Result<Option<CValue>, core::PropError> {
        if method != QTY_STEP {
            return Err(core::PropError::Rejected(format!("{method:?}")));
        }
        let by = args.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
        let v = cx.view(cx.node()).and_then(|n| n.prop_f64(QTY_VALUE)).unwrap_or(0.0) + by;
        cx.set_native(QTY_VALUE, CValue::F64(v))?;
        cx.emit(QTY_CHANGED, Some(CValue::F64(v)));
        Ok(Some(CValue::F64(v)))
    }
}
```

The details that matter:

- `.native()` marks `value` as state that the kind owns and writes itself with `cx.set_native`. Because the kind
  also has a `changed` signal, the generated spec gives `value` a two-way binding on `changed`, so `bind:value`
  works.
- `unit` has no default. It reads as `null` in JavaScript until something sets it.
- `SignalFlags::NONE` makes `changed` a posted signal, delivered after the event, and its Svelte handler receives the
  arguments alone. A `SYNC` signal's handler receives the event object first.
- The hooks you don't write keep their defaults. `measure` returns a zero size, so this leaf is only as big as its CSS
  or its `width` and `height` props make it. A method that `call` doesn't handle is rejected.

`cx.draw_text(text, at, max_width)` in `record` shapes with the document's text shaper. Without one it draws nothing
and returns `None`, so step 4 installs one. `cx.measure_text(text, max_width)`, for `measure`, uses the text system
you pass to `SkelterBackend::with_text`.

### 2. Register the kind with `Kinds::with_app`

`Kinds` is svelters-skelter's table of the kinds that JavaScript can create. `Kinds::with_app` registers
skelter-widgets' kinds, then calls your closure on the same registry, then fills the remaining built-in tags with
placeholders. A kind you register gets its JavaScript-facing spec from its `KindSpec`, on top of the common
`Widget` base, so it also accepts `anchor`, `flex-grow`, `width`, `height`, `enabled`, `tooltip` and the other
common props.

Build the table once, in a `thread_local!` as cadatomic and `svelters_skelter::builtin_kinds()` do, because deriving
the specs leaks them:

```rust
use std::sync::Arc;
use skelter_core::{KindRegistry, RegistryError};
use svelters_skelter::Kinds;

fn register(reg: &mut KindRegistry) -> Result<(), RegistryError> {
    reg.register::<QuantityField>()?;
    Ok(())
}

pub fn app_kinds() -> Arc<Kinds> {
    thread_local! {
        static KINDS: Arc<Kinds> = Arc::new(
            Kinds::with_app(&svelters_skelter::surface::builtin(), register).expect("the app's kinds register"),
        );
    }
    KINDS.with(Arc::clone)
}
```

Pass `svelters_skelter::surface::builtin()` (svelters-skelter's built-in kinds, including the `Widget` base that app
kinds extend) as the first argument. `reg.register::<K>()` needs `K: Kind + Default`. For a kind built from data,
use `reg.register_with(&SPEC, factory)`.

### 3. Give the compiler a spec fragment

The Svelte compiler checks every element against skelter's semantic spec, `skelter.json`, which only knows the
built-in kinds. `svelters_skelter::spec::fragment_text(&kinds)` writes only your app's kinds, as a fragment that
the compiler merges into it.

For a live compile, write the fragment to a file and assign it as the config's fragment list, as the
svelters-skelter tests and cadatomic do:

```rust
let mut cfg = svelters_host::modules::SourceConfig::for_path(app_dir)?;
cfg.backend = "skelter".into();
let fragment = cfg.root.join("app.skelter.json");
std::fs::write(&fragment, svelters_skelter::spec::fragment_text(&kinds)).map_err(|e| e.to_string())?;
cfg.spec_fragments = vec![fragment];
```

`SourceConfig::for_path` already reads `"specFragments"` from the app's `package.json`. That field suits a fragment
written ahead of time, for `svelters-compile` or vite. Don't use both for the same file: the compiler merges every
entry, and the second merge of one fragment fails with `spec fragment: kind QuantityField is already defined`.

To check a component once, run the compiler from the svelters checkout:
`cargo run -p svelters-compiler --bin svelters-compile -- --backend skelter --spec-fragment app.skelter.json --check App.svelte`
(its usage text doesn't list `skelter` yet, but accepts it). Regenerate the fragment whenever a spec changes,
because a member's type is part of its id.

### 4. Run the app with your kinds

The stock runner (`cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry <app dir>` in the svelters
checkout) only knows the built-in kinds, so write your own `main` in your binary crate. It is the runner's code with
`app_kinds()` in place of `builtin_kinds()`, the fragment from step 3, and one line the stock runner lacks: a text
shaper for `draw_text`. Run it from the directory that holds `ui/`.

```rust
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use svelters_skelter::app::SveltersApp;
use svelters_skelter::embed::{Embed, EmbedConfig};
use svelters_skelter::SkelterBackend;

fn main() -> Result<(), String> {
    let kinds = app_kinds();
    let fonts = skelter_text::FontStore::new(skelter_text::FontConfig::default());
    let backend = SkelterBackend::with_text(kinds.clone(), Box::new(skelter_text::TextSystem::new(fonts.clone())));
    svelters_skelter::app::install_widget_services(&backend, skelter_text::TextSystem::new(fonts.clone()));
    // not in the stock runner: a shaper for `RecordCx::draw_text`
    backend.with_doc(|d| d.set_text_shaper(Box::new(skelter_text::TextSystem::new(fonts.clone()))));

    let embed = Embed::with_backend(backend, EmbedConfig::app(svelters_host::HostConfig::default())).map_err(|e| e.to_string())?;
    let mut cfg = svelters_host::modules::SourceConfig::for_path(Path::new("ui"))?;
    cfg.backend = "skelter".into();
    let fragment = cfg.root.join("app.skelter.json");
    std::fs::write(&fragment, svelters_skelter::spec::fragment_text(&kinds)).map_err(|e| e.to_string())?;
    cfg.spec_fragments = vec![fragment];
    let provider = svelters_host::modules::SourceProvider::new(cfg)?;
    embed.load_app_from(Rc::new(provider)).map_err(|e| e.to_string())?;

    let glyphs: Arc<dyn skelter_base::text::GlyphRasterizer> = Arc::new(skelter_text::FontRasterizer::new(fonts));
    let spawn = skelter::compositor(|clock| skelter_compositor::CompositorConfig::new(clock, skelter_base::time::Span::from_micros(16_667)), glyphs);
    skelter_platform::run(Box::new(SveltersApp::new(embed, spawn))).map_err(|e| format!("{e}"))
}
```

`install_widget_services` gives the document what skelter-widgets' kinds need (their text service, the operating
system's interaction policy, the default theme), but no text shaper and no style sheet. If your host already owns a
`skelter_core::Document`, use `SkelterBackend::shared(kinds, doc)` instead, with the document as `Rc<RefCell<Document>>`. Its registry must contain every kind in `kinds` under the same tag: build
the document on `kinds.registry()`, or register into its registry with `svelters_skelter::kinds::register_builtin`
plus your own `register`.

### 5. Use the element in Svelte

Your tag is now an element like the built-ins. Props are attributes, a signal `changed` is the callback prop
`onchanged`, and methods are called on the element handle from `bind:this`. Put this in `ui/src/App.svelte`:

```svelte
<script>
  let field;
  let qty = $state(0);
</script>

<window title="Kinds" default-width={360} default-height={240}>
  <box flex-direction="row" gap={8}>
    <quantity-field bind:this={field} class="qty" unit="mm" width={80} height={24}
      onchanged={(v) => (qty = v)} />
    <button text="+1" onclicked={() => field.call('step', 1)} />
    <label text={String(qty)} />
  </box>
</window>

<style>
  .qty {
    border-radius: 3px;
    border: 1px solid #889;
    background-color: #dde3ea;
    color: #1b1f24;
  }
</style>
```

This component is an illustration, assembled from svelters-skelter's tests (`onchanged`, `el.call(...)`) and
svelters' `examples/egui-kinds` app (the `<window>` root). The runner opens a platform window for
each `<window>` the app presents, sized by `default-width` and `default-height`.

The colours are literal: a kind's box starts transparent, and svelters-skelter adds no sheet that defines the
`--sv-*` theme variables. To use them, add the theme's token sheet in `main` before loading the app, as cadatomic
does with its own theme (this needs `skelter-widgets`):
`backend.with_doc(|d| { d.add_sheet(&skelter_widgets::theme::Theme::default().css()); });`

Enum props take the value's name (`tone="warning"`). A nested object passed from Svelte arrives in Rust as a nested
`Value::Record`. A method that returns `PropError::coded("E_RANGE", "...")` throws an `Error` whose `.code` is
`"E_RANGE"`.

## What you should see

The component compiles without errors. A window shows a light, rounded box reading `0 mm` beside a `+1` button.
Each click calls `step` in Rust, which writes `value` and emits `changed`; the box then reads `1 mm` and the label
shows `1`. In the generated fragment, your kind's entry has `"widget": "app"` and
`"bindings": { "value": { "event": "changed", ... } }`, and your tag is listed under `"tags"`
(`ui/app.skelter.json`).

## Attach Rust code to an existing element with `use:`

For Rust behaviour on an existing element, such as a command run on a click, svelters-skelter routes Svelte
actions to Rust:
`use:run={{ command: 'file.save' }}`, imported from `svelters:behaviors/<ns>`.

1. Implement `svelters_skelter::behavior::Behavior`. `attach` gets the element's node and the params so far,
   `update` gets the whole params record after any change, and `detach` runs when the action is destroyed or the
   element's node is freed. From svelters-skelter's tests (`Run` logs its hooks):

   ```rust
   use skelter_core::{ListenFlags, common as core_common};
   use std::cell::RefCell;
   use std::rc::Rc;
   use svelters_skelter::behavior::{Behavior, BehaviorCx, Props};

   type Calls = Rc<RefCell<Vec<String>>>;

   struct Run(Calls);

   impl Behavior for Run {
       fn attach(&mut self, cx: &mut BehaviorCx<'_>, props: &Props) {
           cx.doc.listen(cx.node, core_common::CLICK, ListenFlags::PASSIVE);
           self.0.borrow_mut().push(format!("attach {:?}", props.get("command")));
       }
       fn update(&mut self, _cx: &mut BehaviorCx<'_>, props: &Props) {
           self.0.borrow_mut().push(format!("update {:?}", props.get("command")));
       }
       fn detach(&mut self, cx: &mut BehaviorCx<'_>) {
           cx.doc.unlisten(cx.node, core_common::CLICK, false);
           self.0.borrow_mut().push("detach".into());
       }
   }
   ```

2. Register it on the backend in `main`, after `install_widget_services` and before `Embed::with_backend` takes the
   backend. Each `use:` makes one instance from the factory:

   ```rust
   let calls: Calls = Rc::default();
   let c = calls.clone();
   backend.register_behavior("cmd", "run", move || Box::new(Run(c.clone())) as Box<dyn Behavior>);
   ```

3. Write the module that Svelte imports. `backend.behavior_modules()` returns `(namespace, source)` pairs, and the
   module loader looks for each one at `<app root>/.svelters/app/behaviors/<ns>.js`. svelters-skelter does not write
   this file for you. Collect the pairs while you still hold `backend` (`let modules = backend.behavior_modules();`
   next to the registration), then write them after `cfg` is built and before `SourceProvider::new(cfg)`:

   ```rust
   // illustrative
   for (ns, src) in modules {
       let dir = cfg.root.join(".svelters/app/behaviors");
       std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
       std::fs::write(dir.join(format!("{ns}.js")), src).map_err(|e| e.to_string())?;
   }
   ```

4. Use it: `import { run } from 'svelters:behaviors/cmd';` and `<button text="Save" use:run={{ command: 'file.save' }} />`.

To react to input without JavaScript, add `backend.add_event_tap(|doc, ev, events| { ... })`. The tap sees every
input the backend dispatches, with the JavaScript-facing events it produced, before JavaScript does. The
`cx.doc.listen` call above makes sure the tap sees the clicks. cadatomic
registers its `command`, `op`, `live` and `presents` behaviours this way.

## Add `:state()` pseudo-classes for your kinds

State bits 0–31 are svelters' built-in states (`:hover`, `:checked`, and so on). Bits 32–63 belong to you. To let CSS
test them:

1. Define the bits and a name table, as cadatomic does:
   `pub const PENDING: StateMask = StateMask(1 << 32);` and
   `pub const STATES: [(&str, StateMask); 1] = [("cad.pending", PENDING)];`
2. Call `backend.set_state_names(Box::new(STATES))` before the app's style sheets load. Sheets parsed earlier don't
   know the names.
3. Set the bit from your kind with `cx.set_state(PENDING, true)` in an event or method, or with
   `doc.set_state(node, PENDING, true)` from the host.
4. Style it: `.slot-row:state(cad.pending) { ... }`.

## When it doesn't work

- **`unknown_element` (or `unknown_property`, `unknown_event`, `unknown_method`) on your tag.** The compiler didn't
  get your fragment, or your tag isn't in it. Check that the path is in `cfg.spec_fragments` (or `specFragments` in
  `package.json`) and that you regenerated it after changing the spec. If your tag is missing from the fragment's
  `"tags"`, your `register` closure did not register it. `svelters-compile --explain unknown_element` describes the
  diagnostic.
- **`spec fragment for backend "egui" cannot extend the skelter spec`.** You passed an egui app's dumped
  `kinds.json`. Generate the fragment with `svelters_skelter::spec::fragment_text` instead.
- **`spec fragment: kind QuantityField is already defined`.** The name is derived from your tag. Usually the same
  fragment was merged twice, from `cfg.spec_fragments` and from `package.json`; list it in one place. It also
  happens when you register a tag that a built-in kind uses, as does **`spec fragment: tag <x> is already
  defined`**; pick a tag that no built-in kind uses.
- **"kind `x` collides with registered kind `y`"** from `Kinds::with_app`: the tag is taken, usually by a
  skelter-widgets kind. **"`k.a` and `other.b` hash to the same id …: rename one"**: one of your members has the same
  id as a member of another kind, so rename your member (or the tag). **"kind `k`: members `a` and `b` have the same
  id"** is a collision inside your own kind.
- **The element compiles but draws nothing.** Your `record` pushes nothing, or the leaf has a zero size, because
  `measure` defaults to zero. Give it a `width` and `height`, or implement `measure`. A box with no `background-color`
  or `border` in your CSS is transparent.
- **The box draws but its text doesn't.** The document has no text shaper. Add the `set_text_shaper` line from step 4.
- **A method call throws `... is not implemented by this kind`.** Your `call` doesn't override the default.
- **`the host document's registry lacks <tag>`** from `SkelterBackend::shared`: build the host's registry as the
  end of step 4 says.
- **`"svelters:behaviors/cmd" -> ...: ... (run the app binary's dump)`.** The behaviour module file is missing. Write
  `behavior_modules()` to `.svelters/app/behaviors/` (step 3 of the `use:` section).
- **A `:state()` rule never matches.** The sheet was parsed before `set_state_names`, or the name isn't in your table.
  In a development build nothing is reported; a precompiled release sheet fails to load with
  "unknown provider state … in :state()".
