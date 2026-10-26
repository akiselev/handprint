# Add a native widget kind to a skelter app and use it from Svelte

This guide adds a widget written in Rust to a svelters app that runs on skelter, and then uses it from a Svelte
component as an ordinary element: `<quantity-field unit="mm" onchanged={...} />`. When you finish, the element
compiles without errors, its props and methods reach your Rust code, and the signals it emits arrive in Svelte as
event callbacks.

Further down the page are two related extension points: attaching Rust code to an existing element with `use:`,
and defining your own `:state()` pseudo-classes for CSS.

## Before you start

You need:

- A svelters checkout. The svelters and skelter crates are not published (`publish = false`), so your application
  depends on them by path, as cadatomic does:
  `svelters-skelter = { path = "../svelters/crates/svelters-skelter" }`. You also need `skelter-core`,
  `skelter-base`, `skelter-text`, `skelter-platform`, `skelter`, `skelter-compositor` and `svelters-host`.
- A Svelte app that already runs on skelter, for example with
  `svelters-skelter-run --entry <app dir>`. Its `package.json` has a `"svelters"` section.
- Familiarity with skelter-core's `Kind` trait. A kind is a Rust type that describes itself with a static
  `KindSpec` and records its display list in `record`.

There are two ways to write the kind. This guide uses skelter-core's `Kind` trait directly, which is what
svelters-skelter's own tests do and what cadatomic uses for `<quantity-field>`, because that kind needs IME input and a
text-input client. skelter-widgets also has a kind interface of its own (the `kind_props!` macro,
`skelter_widgets::shim::core::Kind`, painting through `skelter_widgets::paint`, registered with
`skelter_widgets::host::register`). It gives you skelter-widgets' text painting and theme. cadatomic's
`<status-text>` (`crates/cadatomic-ui/src/skelter/status_text.rs`) is a short example of that interface. Steps 2 to 5
are the same for both.

## Steps

### 1. Write the kind

A kind is a `KindSpec` and a type that implements `Kind`. The spec lists the props, signals and methods that Svelte
can use. Each member's id is a hash of the kind's tag, the member's name and its type name (`number` for `F64`,
`string` for `Str`, and so on), so the constants you match on are built with `PropId::of`, `SignalId::of` and
`MethodId::of`.

This kind is adapted from svelters-skelter's test of app kinds. The only change is the `cx.draw_box()` call, which
draws the element's CSS box (background, border, outline) so you can see it:

```rust
use skelter_base::accesskit::Role;
use skelter_base::change::ChangeClass;
use skelter_core::{self as core, ArgSpec, EventCx, Kind, MethodId, MethodSpec, PropDefault, PropId, PropSpec, RecordCx, SignalFlags, SignalId, SignalSpec, Value as CValue, ValueType};

const QTY_VALUE: PropId = PropId::of("quantity-field", "value", "number");
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
        cx.draw_box(); // added: the CSS box
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

To draw text, use `cx.draw_text(text, at, max_width)` in `record` and `cx.measure_text(text, max_width)` in
`measure`. Both use the node's computed font.

### 2. Register the kind with `Kinds::with_app`

`Kinds` is svelters-skelter's table of the kinds that JavaScript can create. `Kinds::with_app` registers
skelter-widgets' kinds first, then calls your closure on the same registry, and then fills the remaining built-in
tags with placeholders. A kind you register gets its JavaScript-facing spec from its `KindSpec`, on top of the common
`Widget` base, so it also accepts `anchor`, `flex-grow`, `width`, `height`, `enabled`, `tooltip` and the other
common props.

Build the table once and share it. cadatomic keeps it in a `thread_local!`, as `svelters_skelter::builtin_kinds()`
does, because deriving the specs leaks them:

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

Pass `svelters_skelter::surface::builtin()` (the tier 0 and tier 1 built-ins) as the first argument. That list
includes the `Widget` base that app kinds extend. `reg.register::<K>()` needs `K: Kind + Default`. For a kind built
from data, use `reg.register_with(&SPEC, factory)`.

### 3. Give the compiler a spec fragment

The Svelte compiler checks every element against skelter's semantic spec, `skelter.json`, which only knows the
built-in kinds. `svelters_skelter::spec::fragment_text(&kinds)` writes your app's kinds, and only those, as a
fragment that the compiler merges into the built-in spec.

For a live compile from source, write the fragment to a file and add it to the `SourceConfig`. This is the pattern
the svelters-skelter tests and cadatomic both use:

```rust
let mut cfg = svelters_host::modules::SourceConfig::for_path(app_dir)?;
cfg.backend = "skelter".into();
let fragment = cfg.root.join("app.skelter.json");
std::fs::write(&fragment, svelters_skelter::spec::fragment_text(&kinds)).map_err(|e| e.to_string())?;
cfg.spec_fragments.push(fragment);
```

`SourceConfig::for_path` also reads fragments from the app's `package.json`, relative to the app's root:

```json
{ "svelters": { "toolkit": "skelter", "specFragments": ["app.skelter.json"] } }
```

For a one-off compile, `svelters-compile --backend skelter --spec-fragment app.skelter.json App.svelte` does the same
merge. The fragment is derived from your `KindSpec`s. Regenerate it whenever a spec changes, because a member's type
is part of its id.

### 4. Run the app with your kinds

The stock `svelters-skelter-run` binary only knows the built-in kinds, so write your own `main`. This one is the
runner's code with `builtin_kinds()` replaced by `app_kinds()` and the fragment from step 3 added:

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

    let embed = Embed::with_backend(backend, EmbedConfig::app(svelters_host::HostConfig::default())).map_err(|e| e.to_string())?;
    let mut cfg = svelters_host::modules::SourceConfig::for_path(Path::new("ui"))?;
    cfg.backend = "skelter".into();
    let fragment = cfg.root.join("app.skelter.json");
    std::fs::write(&fragment, svelters_skelter::spec::fragment_text(&kinds)).map_err(|e| e.to_string())?;
    cfg.spec_fragments.push(fragment);
    let provider = svelters_host::modules::SourceProvider::new(cfg)?;
    embed.load_app_from(Rc::new(provider)).map_err(|e| e.to_string())?;

    let glyphs: Arc<dyn skelter_base::text::GlyphRasterizer> = Arc::new(skelter_text::FontRasterizer::new(fonts));
    let spawn = skelter::compositor(|clock| skelter_compositor::CompositorConfig::new(clock, skelter_base::time::Span::from_micros(16_667)), glyphs);
    skelter_platform::run(Box::new(SveltersApp::new(embed, spawn))).map_err(|e| format!("{e}"))
}
```

`install_widget_services` gives the document what skelter-widgets' kinds need: text shaping, the operating system's
interaction policy and the default theme. If your host already owns a `skelter_core::Document`, use
`SkelterBackend::shared(kinds, doc)` instead. That document's registry must contain every kind in `kinds` under the
same tag, which `svelters_skelter::kinds::register_builtin` adds.

### 5. Use the element in Svelte

Your tag is now an element like the built-ins. Props are attributes, a signal `changed` is the callback prop
`onchanged`, and methods are called on the element handle from `bind:this`:

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
    background-color: var(--sv-list-hover);
  }
</style>
```

This component is an illustration, assembled from the patterns in svelters-skelter's tests (`onchanged`,
`el.call(...)`), svelters' `examples/egui-kinds` app (the `<window>` root) and cadatomic's panels (class rules with
theme variables). The runner opens a platform window for each `<window>` the app presents, sized by
`default-width` and `default-height`.

Enum props take the value's name as a string (`tone="warning"`). Record and list props and method arguments keep
their shape: a nested object passed from Svelte arrives in Rust as a nested `Value::Record`. If a method returns
`PropError::coded("E_RANGE", "...")`, JavaScript sees an `Error` whose `.code` is `"E_RANGE"`.

## What you should see

The component compiles with no diagnostics. A window shows a rounded box beside a `+1` button. Each click calls
`step` in Rust, which writes `value`, emits `changed`, and the label shows the new number. In the generated fragment,
your kind's entry has `"widget": "app"` and `"bindings": { "value": { "event": "changed", ... } }`. You can check
that by opening the file you wrote in step 3.

## Attach Rust code to an existing element with `use:`

You don't always need a whole new kind. Sometimes you only want Rust behaviour on an element that already exists, for
example a command run when a button is clicked. For that, svelters-skelter routes Svelte actions to Rust:
`use:run={{ command: 'file.save' }}`, imported from `svelters:behaviors/<ns>`.

1. Implement `svelters_skelter::behavior::Behavior`. `attach` gets the element's node and the params so far,
   `update` gets the whole params record after any change, and `detach` runs when the action is destroyed or the
   element's node is freed. This is from svelters-skelter's tests:

   ```rust
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

2. Register it on the backend. Each `use:` makes one instance from the factory:
   `backend.register_behavior("cmd", "run", move || Box::new(Run(c.clone())) as Box<dyn Behavior>);`
3. Write the module that Svelte imports. `backend.behavior_modules()` returns `(namespace, source)` pairs, and the
   module loader looks for each one at `<app root>/.svelters/app/behaviors/<ns>.js`. svelters-skelter does not write
   this file for you. Write it before you load the app:

   ```rust
   // illustrative
   for (ns, src) in backend.behavior_modules() {
       let dir = cfg.root.join(".svelters/app/behaviors");
       std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
       std::fs::write(dir.join(format!("{ns}.js")), src).map_err(|e| e.to_string())?;
   }
   ```

4. Use it: `import { run } from 'svelters:behaviors/cmd';` and `<button text="Save" use:run={{ command: 'file.save' }} />`.

To react to input without any JavaScript, add `backend.add_event_tap(|doc, ev, events| { ... })`. The tap sees every
input the backend dispatches, together with the JavaScript-facing events it produced, before JavaScript does. A
behaviour that listens on its node (the `cx.doc.listen` call above) makes sure the tap sees the clicks. cadatomic
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
  get your fragment. Check that the path is in `cfg.spec_fragments` (or `specFragments` in `package.json`) and that
  you regenerated it after changing the spec. `svelters-compile --explain unknown_element` describes the diagnostic.
- **`spec fragment for backend "egui" cannot extend the skelter spec`.** You passed an egui app's dumped
  `kinds.json`. Generate the fragment with `svelters_skelter::spec::fragment_text` instead.
- **`spec fragment: kind ... is already defined`.** A tag in your fragment is already in the built-in spec. Pick a
  tag that no built-in kind uses.
- **`kind `x` collides with registered kind `y``** from `Kinds::with_app`: the tag is taken, usually by a
  skelter-widgets kind. **`... hash to the same id ...: rename one`**: two member ids collide, so rename one of the
  members.
- **The element compiles but draws nothing.** Your `record` pushes nothing, or the leaf has a zero size, because
  `measure` defaults to zero. Give it a `width` and `height`, or implement `measure`. If the fragment says
  `"widget": "placeholder"` for your tag, your `register` closure never ran for it.
- **A method call throws `... is not implemented by this kind`.** Your `call` doesn't override the default.
- **`the host document's registry lacks <tag>`** from `SkelterBackend::shared`: build the host's registry with the
  same kinds (`register_builtin` plus your `register`).
- **`"svelters:behaviors/cmd" -> ...: ... (run the app binary's dump)`.** The behaviour module file is missing. Write
  `behavior_modules()` to `.svelters/app/behaviors/` (step 3 of the `use:` section).
- **`unknown provider state `x` in :state()`.** The sheet was parsed before `set_state_names`, or the name isn't in
  your table.
