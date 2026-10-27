# A Svelte counter in a native window, from the inside

In this tutorial you build a small app: a Svelte 5 counter in a native desktop window. First you run it
with the runner that ships with svelters, then from a Rust binary of your own. The app is only the vehicle.
What you are really here for is what happens underneath: how a `.svelte` file turns into nodes of a Rust
tree, how a click gets from the operating system to your handler, how `count++` turns into new pixels, and
where the Svelte side ends and the Rust side begins. At several points you'll stop and change something on
purpose to watch that machinery work.

Everything here runs inside a checkout of the svelters repository. Nothing is published to crates.io or npm
yet, and the steps have been run end to end on Linux with X11 only.

## What you are building into

The problem svelters solves is this: you want to write your UI as Svelte components, but you want a native
window, not a browser. There is no webview and no Electron in the process. Instead, the pieces are:

- **Your Rust process.** Its main thread runs the platform event loop (winit) and everything that touches
  the UI, JavaScript included.
- **QuickJS**, a small embeddable JavaScript engine (QuickJS-NG, through the `svelters-host` crate). It runs
  your compiled components and Svelte's own reactivity code, on that same main thread.
- **The module loader and compiler.** When JavaScript imports a `.svelte` file, the loader compiles it right
  then, with svelters' Rust port of the Svelte compiler. There's no Vite and no Node at run time.
- **The native runtime**, `@svelters/native-runtime`. This is the JavaScript library compiled components call.
  Where Svelte in a browser would call `document.createElement`, this one asks the host to create a node of
  kind `button`, or to set a node's `text` prop.
- **The tree.** skelter is svelters' own GUI toolkit, and its core (`skelter-core`) holds one retained tree
  of nodes in Rust. Every node has a **kind**: a Rust type that decides how the node lays out, paints, takes
  input and appears to screen readers.
- **The compositor**, a separate thread that owns the GPU (through wgpu) and draws whatever the main thread
  last committed.

```text
 your binary, one process
 ┌───────────────────────────── main thread ──────────────────────────────┐
 │                                                                         │
 │  winit ──input──▶  skelter-core tree  ◀──host ops──  QuickJS            │
 │  event loop        (nodes of kinds)   ──events────▶  your compiled      │
 │                          │                           Svelte + runtime   │
 └──────────────────────────┼──────────────────────────────────────────────┘
                            │ commit: what changed this frame
                            ▼
              compositor thread ── wgpu ──▶ pixels in the window
```

Keep the two arrows between the tree and QuickJS in mind. JavaScript reaches the tree only through **host
ops**: create a node, insert it under another, set a prop, listen to a signal. The tree reaches JavaScript
only through **events**, and only the ones JavaScript listens to. The rest of this page fills in what travels
along each arrow, and why.

## Setting up

You need:

- a checkout of `https://github.com/akiselev/svelters`;
- Rust 1.97.0 (the checkout's `rust-toolchain.toml` pins it, so `rustup` installs it on first use);
- Node.js 22 or newer and pnpm 10, only to install packages;
- a C compiler, because QuickJS is built from C sources;
- on Linux, an X11 or Wayland session, libxkbcommon (plus libxkbcommon-x11 on X11) and a Vulkan driver.
  On Debian or Ubuntu, `sudo apt install libxkbcommon-x11-0 mesa-vulkan-drivers` covers the last two; the
  Mesa package includes lavapipe, a software Vulkan driver that works without a GPU.

From the root of the checkout, install the JavaScript packages:

```sh
pnpm install
```

It takes a few seconds. pnpm prints a boxed warning that it ignored esbuild's build script; nothing here
needs esbuild.

Why does a native app need `node_modules` at all? Because the reactivity your components use is Svelte's
own: the native runtime imports Svelte 5.57.2's signals and effects directly, and the module loader reads
those files out of `node_modules` when the app starts. Node itself never runs your app; it only installed the
files.

## A window from a component

Make a directory for the app. Any directory with a `package.json` works; this page uses
`examples/hello-skelter/app`.

```sh
mkdir -p examples/hello-skelter/app/src
```

`examples/hello-skelter/app/package.json`:

```json
{
  "name": "hello-skelter",
  "private": true,
  "type": "module",
  "svelters": { "toolkit": "skelter" }
}
```

`examples/hello-skelter/app/src/App.svelte`:

```svelte
<window title="Hello skelter" default-width={360} default-height={200}>
  <box gap={8}>
    <label>Hello from Svelte</label>
  </box>
</window>
```

Run it from the checkout root:

```sh
cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry examples/hello-skelter/app
```

The first build compiles a few hundred crates; on four cores with debug info off
(`CARGO_PROFILE_DEV_DEBUG=0`) it takes about three minutes. Then a 360 by 200 window titled "Hello skelter"
opens. It is black, with "Hello from Svelte" in light grey in the top-left corner. The black is not a
rendering failure, and you'll see later exactly why it's black. Close the window and the process exits.

Here is what happened between `cargo run` and those pixels, in order:

1. **The runner set up the tree.** Its `main` made one font store, then a `SkelterBackend`. The backend owns
   an empty skelter-core `Document`, which is the tree, and knows the built-in kinds. It then installed what
   the kinds need: text shaping, the operating system's interaction rules, and a theme.
2. **It started QuickJS on the main thread**, attached to that backend. From now on, every host op your
   JavaScript makes is a direct Rust function call into the backend, which edits the `Document`. Nothing is
   serialised or queued in between.
3. **It loaded your app from source.** It read your `package.json` to find the app root and picked
   `src/App.svelte` as the entry. Then it evaluated a small generated entry module equivalent to
   `import App from './src/App.svelte'; mount(App, { props: {} })`. When QuickJS reached that import, it asked
   the module loader for the file, and the loader ran the svelters compiler on it right then, in Rust, and
   handed back JavaScript.
4. **Mounting built the tree.** The compiled component called the native runtime, and the runtime made host
   ops: create a `window` node, a `box`, a `label`; set the window's `title`, `default-width` and
   `default-height`; set the label's `text`; insert the label into the box and the box into the window.
   Finally it called the window's `present` method. At this point no operating-system window exists yet: the
   whole tree sits in the `Document`, detached.
5. **The event loop started.** The runner handed control to winit. Its app handler saw the presented window,
   read `title`, `default-width` and `default-height` back out of the node's props, and asked the platform for
   a window of that size. When the platform reported the window's first size, the handler made your `window`
   node that window's root.
6. **A frame ran.** On the main thread, skelter-core computed styles, laid the tree out with taffy (a flexbox
   engine), asked each dirty node's kind to record a display list (a few rectangles and a text run), and built
   the hit list that input will use. It packed what changed into a **commit** and sent it to the compositor
   thread, which drew it with wgpu and presented it.

Two ideas from that list are the heart of skelter.

The first is **retained**. The tree you built in step 4 stays in memory for the life of the window. Nothing
rebuilds it each frame. A frame only re-records the nodes something has changed, and the commit carries only
those nodes' display lists. Immediate-mode toolkits such as egui work the other way: they run your UI code
and rebuild every visible widget on every frame. Before skelter, svelters drew this kind of app with egui,
and that per-frame cost is the main reason skelter exists.

The second is **kinds**. `<window>`, `<box>` and `<label>` are not HTML elements. Each tag names a kind
registered in Rust. The kind's spec lists its props (each with a type and a change class, which comes up
again below), its signals and its slots. The kind's code measures it, paints it, hit-tests it and handles its
input. `box`, `label` and `button` come from the `skelter-widgets` crate. In a browser the DOM plays this
role. Here there is no DOM, and no virtual DOM either: Svelte never needed one, since it compiles your
markup into direct calls, and svelters points those calls at the Rust tree instead of at a document.

## Experiment: break it on purpose

Change the label to a tag that doesn't exist on skelter. `<push-button>` is a widget name from GTK and Qt:

```svelte
<window title="Hello skelter" default-width={360} default-height={200}>
  <box gap={8}>
    <push-button>Hello from Svelte</push-button>
  </box>
</window>
```

Run the same command. No window opens; the process exits with status 1 and prints something like this
(your path will differ):

```text
Load: Error loading module '/home/you/svelters/examples/hello-skelter/app/src/App.svelte': /home/you/svelters/examples/hello-skelter/app/src/App.svelte:3:4: error[unknown_element] (skelter): <push-button> is not an element of the skelter backend
  help: did you mean one of `button`, `radio-button`, `toggle-button`?
```

Read it from the left. `Load: Error loading module '…'` is QuickJS: it asked the loader for your module and
got an error back instead of code. Everything after the second path is the compiler's diagnostic: the file,
line and column; the code `unknown_element`; the backend it compiled for; and a `help:` line with the nearest
names it knows.

How does the compiler know every tag skelter has? It compiles against a spec,
`metadata/semantics/skelter.json`, which is generated from the kinds registered in Rust. The spec lists every
tag, every prop with its type, and every signal. So the mistake is caught when the file is compiled, at
startup, with a suggestion, instead of turning into a node that silently draws nothing. The same spec catches
an event the element doesn't have: change the tag back and add `onfoo={() => {}}` to the label, and you get
`error[unknown_event] (skelter): <label> has no event `foo` on skelter`.

One warning about this message: it reaches you through QuickJS's module-loading error, which shortens it to
about 256 characters. Your file's path appears twice, so in a deeply nested checkout you may see only the
path. The compiler is also a command-line tool, and it prints the whole diagnostic with the path as you typed
it:

```sh
cargo run -p svelters-compiler --bin svelters-compile -- --backend skelter examples/hello-skelter/app/src/App.svelte
```

Put the `<label>` back before you go on.

## Making it react

Replace `App.svelte` with a counter:

```svelte
<script>
  let count = $state(0);
  let doubled = $derived(count * 2);

  $effect(() => {
    console.log('count is', count);
  });
</script>

<window title="Hello skelter" default-width={360} default-height={200}>
  <box gap={8}>
    <label>Count: {count}</label>
    <label>Doubled: {doubled}</label>
    <button onclicked={() => count++}>Increment</button>
  </box>
</window>
```

Run it again. The runner doesn't watch your files, so every edit needs a restart, and every restart compiles
the app again; for a component this size you won't notice. The window shows "Count: 0", "Doubled: 0" and an
**Increment** button as wide as the window. Click it twice and the labels read "Count: 2" and "Doubled: 4".
The terminal shows:

```text
[log] count is 0
[log] count is 1
[log] count is 2
```

The host echoes `console.log` to stderr as `[log] …`. Notice when the first line appeared: before the window
did. The `$effect` ran while the app was loading (step 4 above), when the tree was still detached and the
event loop hadn't started.

`$state`, `$derived` and `$effect` are ordinary Svelte 5. What's new is the attribute `onclicked`. It is `on`
followed by the name of a signal of the `button` kind, `clicked`, which the spec describes as "pressed and
released". Here is what a click does on its way to your arrow function:

1. **The platform reports a pointer press and a release.** winit delivers them to the main thread, and each
   goes into skelter-core's dispatch as one transaction.
2. **Hit testing.** skelter-core finds the node under the pointer using the hit list built by the last
   frame, so a click lands on what that frame put on screen.
3. **Native defaults.** The button's kind handles the input in Rust. It tracks hover and press itself, and
   its pressed look comes from style rules on those states. On the release over the button, it emits its
   `clicked` signal.
4. **Only listened signals become events.** The compiled code registered a listener for `clicked` when it
   built the button, so skelter-core lists a JavaScript-facing event. `press`, `release` and every pointer
   move have no listener, so they never leave Rust.
5. **Delivery at event time.** svelters-skelter posts the event to the host and runs a host checkpoint
   before the input's handling ends. QuickJS calls the runtime's dispatch function, which finds your handler
   by the node's handle and calls it. `count++` runs.
6. **The microtasks drain** inside that same checkpoint. That is where Svelte runs the effects that read
   `count`.

Then the new number has to reach the screen. Svelte's reactivity is fine-grained: `count++` doesn't re-render
the component. It marks the signals that depend on `count` and schedules the effects that read them. The
labels' text is written by one template effect, which re-runs and computes `"Count: 1"` and `"Doubled: 2"`.
The runtime compares each value with the last one it wrote to that node; only the changed ones cross into
Rust, as `setProp` calls on the two labels' handles. In skelter-core, `set_prop` stores the value and marks
the work the prop's change class calls for. A label's `text` is a layout-class prop, so the label is
re-measured. Layout runs again only if its size changed, and the label is marked for repainting. At the next
frame, only those two labels re-record their display lists. The commit carries just those two lists, and the
compositor redraws the region that changed. The button and the box are not touched.

## Experiment: read what the compiler made

You've seen the compiler's error output; now look at its normal output for the counter:

```sh
cargo run -p svelters-compiler --bin svelters-compile -- --backend skelter examples/hello-skelter/app/src/App.svelte > /tmp/App.js
```

The JavaScript goes to stdout, so it lands in `/tmp/App.js`. It's short. Look for these:

- `$n.toplevel("window", …)` creates the window node. `$n` is the native runtime, and `$` is the reactive
  core.
- A `$n.template(…)` table at module scope, and a `$n.instantiate(…)` call in the component. The compiler
  saw that the box, two labels and button form a subtree whose shape never changes, and described it once as
  data.
- Constants such as `const $$P_Label_text = 135398928;` and one `$n.ids("skelter", …)` call. Props are
  named by numbers, not strings. Search `skelter.json` for `135398928` and you'll find it under `Label` →
  `props` → `text`, next to `"change": "layout"`. The compiler took the id from the spec, the Rust side
  computes the same hash, and the `$n.ids` call checks once per module that the two agree.
- A `$.template_effect(…)` whose body writes the two labels' text with `$n.setp(…, $$P_Label_text, …)`.
  That is the one effect that re-runs on a click.
- `$.update(count)` inside the button's listener (registered with `$n.template_listen` or `$n.on`), and
  `$.user_effect(…)` for your `$effect`.

What you won't find is anything resembling HTML, a `render()` function, or a virtual node. The component is
a function that makes host calls once, plus effects that make more host calls when the signals they read
change.

A detail about the template: it is an optimisation the skelter spec advertises (its `templates`
capability), meant to let the backend build the whole subtree in one call. svelters-skelter doesn't build
templates natively yet, so today the runtime expands the table into one `create`, `setProp` and `insert`
per node, the same calls the compiler would have written without templates. The spec describes what the
compiler may emit; the backend decides what it can do in one step.

## Experiment: watch the order of events

Now make the order of events visible. Replace the script and the two labels so that every step logs:

```svelte
<script>
  let count = $state(0);
  let doubled = $derived(count * 2);

  function show(what, value) {
    console.log('template reads', what, value);
    return value;
  }

  function increment() {
    console.log('handler: count was', count);
    count++;
    console.log('handler: count is now', count);
  }

  $effect(() => {
    console.log('effect: count is', count);
  });
</script>

<window title="Hello skelter" default-width={360} default-height={200}>
  <box gap={8}>
    <label>Count: {show('count', count)}</label>
    <label>Doubled: {show('doubled', doubled)}</label>
    <button onclicked={increment}>Increment</button>
  </box>
</window>
```

Before you run it, predict what one click prints. Do both handler lines come before any template line, or
does `count++` update the labels in the middle of the handler? Does the effect come before or after the
template?

Then run it and click once. From the order the host delivers events and the way Svelte schedules effects,
you should see:

```text
[log] handler: count was 0
[log] handler: count is now 1
[log] template reads count 1
[log] template reads doubled 2
[log] effect: count is 1
```

The handler runs to the end first: `count++` only marks things dirty, and the second `console.log` reads the
new value directly from the signal. Svelte runs the effects afterwards, in the microtask drain of the same
checkpoint: the template effect first (it writes the labels), then your `$effect`, which runs after the tree
has been updated.

Now add a third label that reads something the click doesn't change:

```svelte
<script>
  // ...as before, plus:
  let name = $state('Ada');
</script>

<!-- ...inside the box, after the second label: -->
    <label>Name: {show('name', name)}</label>
```

Predict again: does a click log `template reads name`? It shouldn't. Each function call in a template is
compiled into a derived value of its own, which re-computes only when what it read changes. `name` didn't
change, so `show('name', …)` isn't called. The cheaper case behaves the same way for a different reason.
With a plain `{name}`, the template effect does re-run its expression, but the runtime sees the same string
it wrote last time and sends nothing to Rust. That is what "fine-grained" means here: work happens only
where a value changed, first in JavaScript and then again in Rust, where `set_prop` returns at once if the
stored value is unchanged.

## Hosting it from your own binary

The runner is about seventy lines of Rust, and your own host can start as the same code. Make
`examples/hello-skelter` a crate with this `Cargo.toml`:

```toml
[package]
name = "hello-skelter"
edition.workspace = true
version.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
skelter = { workspace = true }
skelter-base = { workspace = true }
skelter-compositor = { workspace = true }
skelter-platform = { workspace = true }
skelter-text = { workspace = true }
svelters-host = { workspace = true }
svelters-skelter = { workspace = true }
```

The workspace lists its members by name, so add it to `members` in the root `Cargo.toml`:

```toml
members = [
    # ...
    "examples/egui-kinds",
    "examples/hello-skelter",
]
```

`examples/hello-skelter/src/main.rs` is the runner's `main` without the argument parsing, with the app
directory fixed to the `app` folder next to the crate:

```rust
use std::sync::Arc;
use svelters_skelter::app::SveltersApp;
use svelters_skelter::embed::{Embed, EmbedConfig};
use svelters_skelter::SkelterBackend;

fn main() {
    let fail = |e: String| -> ! {
        eprintln!("{e}");
        std::process::exit(1)
    };
    let entry = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("app");

    // text: one font store for measuring (the tree), shaping (the kinds) and rasterising (the GPU)
    let fonts = skelter_text::FontStore::new(skelter_text::FontConfig::default());
    let backend = SkelterBackend::with_text(svelters_skelter::builtin_kinds(), Box::new(skelter_text::TextSystem::new(fonts.clone())));
    svelters_skelter::app::install_widget_services(&backend, skelter_text::TextSystem::new(fonts.clone()));

    let host = svelters_host::HostConfig::default();
    let embed = Embed::with_backend(backend, EmbedConfig::app(host)).unwrap_or_else(|e| fail(e.to_string()));
    let mut cfg = svelters_host::modules::SourceConfig::for_path(&entry).unwrap_or_else(|e| fail(e));
    cfg.backend = "skelter".into();
    let provider = svelters_host::modules::SourceProvider::new(cfg).unwrap_or_else(|e| fail(e));
    if let Err(e) = embed.load_app_from(std::rc::Rc::new(provider)) {
        fail(e.to_string());
    }

    let glyphs: Arc<dyn skelter_base::text::GlyphRasterizer> = Arc::new(skelter_text::FontRasterizer::new(fonts));
    let spawn = skelter::compositor(|clock| skelter_compositor::CompositorConfig::new(clock, skelter_base::time::Span::from_micros(16_667)), glyphs);
    if let Err(e) = skelter_platform::run(Box::new(SveltersApp::new(embed, spawn))) {
        fail(format!("hello-skelter: {e}"));
    }
}
```

Run it with `cargo run -p hello-skelter`. You get the same window and the same log lines, now from a
binary you own.

Read the code against the model. Each block is one of the pieces from the start of the page:

- **The tree and its kinds.** `SkelterBackend::with_text(builtin_kinds(), …)` makes the `Document` on the
  registry of built-in kinds. The text system it receives measures text for layout.
  `install_widget_services` gives the document what the kinds read while they work: text shaping, the
  interaction policy for the current OS (how a press or a drag behaves), and the theme. One `FontStore` is
  shared three ways: by the tree for measuring, by the kinds for shaping and by the compositor for
  rasterising glyphs, so all three agree on the fonts.
- **QuickJS.** `Embed::with_backend` starts the JavaScript host on the backend. `HostConfig::default()` is
  where console echo comes from, and it also sets a 10-second budget for a single entry into JavaScript, so
  an endless loop in a handler is interrupted instead of freezing the window forever.
- **The loader and compiler.** `SourceConfig::for_path` reads `package.json`. Setting `cfg.backend` to
  `"skelter"` compiles against skelter's spec whatever the `toolkit` field says. `load_app_from` compiles and
  evaluates the app, which mounts the component and presents its window, all before any window exists.
- **The compositor.** `skelter::compositor` prepares the compositor thread; it starts once the platform is
  up. The `Span` is the frame interval new windows assume until the platform reports the display's real one
  (16,667 µs is 60 Hz).
- **The event loop.** `skelter_platform::run` runs winit on this thread with `SveltersApp` as its handler.
  That handler opens a window for each `<window>` the app presents, and the loop ends when the last one
  closes.

## Experiment: look at the tree, and count the crossings

Because you own `main` now, you can look at the tree directly. Add this function to `main.rs`:

```rust
/// Prints the tree under each window the app presented: kind, handle, and the text prop if it has one.
fn dump(backend: &SkelterBackend) {
    for h in backend.windows() {
        let Some(root) = backend.node(h) else { continue };
        backend.with_doc(|d| {
            let mut stack = vec![(root, 0)];
            while let Some((n, depth)) = stack.pop() {
                let entry = d.kind_of(n).and_then(|k| d.registry().get(k));
                let kind = entry.map_or("?", |e| e.spec.name);
                let mut line = format!("{}<{}> handle {}", "  ".repeat(depth), kind, backend.handle(n).unwrap_or(0));
                if let Some(e) = entry {
                    if let Some(p) = e.text_prop {
                        let class = e.prop(p).map(|s| s.change);
                        line += &format!(" text={:?} change={:?}", d.get_prop(n, p), class);
                    }
                }
                eprintln!("{line}");
                for &c in d.children(n).iter().rev() {
                    stack.push((c, depth + 1));
                }
            }
        });
    }
}
```

Then make three changes in `main`. Keep a second handle to the backend before `Embed::with_backend` takes it
(`SkelterBackend` is a cheap, shared handle); dump the tree once the app has loaded; and print the backend's
traffic counters after the event loop ends:

```rust
    svelters_skelter::app::install_widget_services(&backend, skelter_text::TextSystem::new(fonts.clone()));
    let probe = backend.clone();

    // ...unchanged...

    if let Err(e) = embed.load_app_from(std::rc::Rc::new(provider)) {
        fail(e.to_string());
    }
    dump(&probe);

    // ...unchanged...

    if let Err(e) = skelter_platform::run(Box::new(SveltersApp::new(embed, spawn))) {
        fail(format!("hello-skelter: {e}"));
    }
    eprintln!("{:?}", probe.traffic());
```

Run it. Before the window appears, you should see the tree your component built, roughly like this (the
handle numbers count up in creation order):

```text
<window> handle 1
  <box> handle 2
    <label> handle 3 text=Some(Str("Count: 0")) change=Some(Layout)
    <label> handle 4 text=Some(Str("Doubled: 0")) change=Some(Layout)
    <button> handle 5 text=Some(Str("Increment")) change=Some(Layout)
```

This is the retained tree as Rust sees it, and it exists before any window does. The handles are the
numbers JavaScript holds. JavaScript never holds a pointer into the tree, only these numbers, which the
backend maps to nodes and never reuses. That is why a handle to a destroyed node can't reach a new one by
accident. `change=Layout` is the change class of each text prop: writing it re-measures the node.

The counters are the other half. The backend counts every call across the boundary, in both directions. Run
the binary three times:

1. close the window without touching it;
2. click **Increment** twice, then close;
3. move the pointer over the button and off it a few times without clicking, then close.

Each run prints a line like `Traffic { syncs: 0, posts: …, changes: 0, checkpoints: …, frames: 0, creates: …,
tree: …, sets: …, gets: …, calls: …, listens: …, styles: … }`. The first run gives you the startup cost:
`creates` and `tree` are the nodes built and inserted, `listens` is the one `clicked` listener, `calls`
includes `present`, and `gets` includes the window's title and size being read back for the platform.

Before you compare the runs, predict. From the click path above:

- two clicks should add 2 to `posts` (one `clicked` event per click) and 4 to `sets` (two label texts per
  click) compared with the first run;
- hovering should add nothing to `posts` or `sets`, even though the button visibly changes as the pointer
  enters it. Hover is handled entirely by the button's kind in Rust, and no listener asked for pointer events.

Ignore `checkpoints`: the embed runs a host checkpoint after every platform callback, so it grows with mouse
movement and timers whatever your app does. If you kept the `name` label from the last experiment, it adds
nothing per click either.

## Why the window is black

You can now explain the black background. Three facts combine:

- **Nothing paints the window.** skelter-widgets has no widget for `window` yet. svelters-skelter fills the
  gap with a placeholder kind: its props are real tree members (the runner read `title` and the default size
  from them), but it records nothing. It's also why the window's documented default padding of 8 doesn't
  show and your text sits flush in the corner: layout comes from CSS, and no rule gives the window padding.
- **The compositor clears to transparent.** Where nothing draws, the compositor's default clear colour is
  transparent. On an ordinary opaque window that shows as black.
- **The theme only supplies colours.** The runner installs VS Code's Dark Modern theme. Kinds use its
  colours for their own parts: the label draws light grey text and the button its fill and border. A theme
  doesn't draw a backdrop by itself.

To get a surface, something that paints has to cover the window. `box` paints its own CSS box (background,
border, shadow), so give it a class and a rule. Add this to `App.svelte`, and change the box to
`<box class="panel" gap={8}>`:

```svelte
<style>
  .panel {
    background-color: #2b2b2b;
    padding: 12px;
  }
</style>
```

The compiler scopes the rule to this component, and the loader hands the sheet to the backend. skelter-core
runs svelters' Rust CSS engine over the tree itself, so `<style>` works without any build step. Predict what
you'll see, then run: a grey panel as wide as the window (a box stretches across its parent, which is a
column flexbox like every node) but only as tall as its content, with black below it. Now add
`flex-grow: 1;` to the rule. The panel should fill the window, because the window root is laid out at the
window's size and the box is its only child.

To separate theme from paint completely, try a light theme with no panel rule. Add
`skelter-widgets = { workspace = true }` to your crate's dependencies and replace the
`install_widget_services` line with the same installation using `Theme::light_modern()`:

```rust
    let policy = *skelter_widgets::interact::Native::for_os(skelter_widgets::interact::Platform::Linux).policy();
    backend.with_doc(|d| skelter_widgets::host::install(d, skelter_text::TextSystem::new(fonts.clone()), policy, skelter_widgets::theme::Theme::light_modern()));
```

The text should turn dark, the button light, and the background stay black. Dark text on black is hard to
read, which is the point of the experiment: the theme changed the kinds' colours, and still nothing painted
the window.

## Where Svelte stops and Rust starts

By now the dividing line should be visible. Svelte decides **what exists** and **what values props have**:
which nodes, in what order, with what text, and what to do when a signal arrives. Rust kinds do **the work
that happens every frame or every pointer move**: layout, painting, hit testing, hover and press, focus,
scrolling, caret movement. That split is deliberate. Direct manipulation never waits on JavaScript, so a busy
handler can't make hover or scrolling stutter. Crossing the boundary costs a function call through QuickJS,
so the design keeps it to the moments where your app actually has to decide something: a click, a changed
value.

The tree also has one owner per value. Some props are owned by the kind while you interact with it, such as
a slider's value during a drag or a scroll offset. If JavaScript writes one of those after the kind has
changed it, and the write is based on an older event, skelter-core refuses the write and the native value
stands. Stale JavaScript can't paint over a live drag.

When an app needs more than the built-in kinds, it extends the Rust side at three points:

- **Kinds.** An app registers its own Rust kinds alongside the built-in ones (`Kinds::with_app`). They become
  tags your components can use, and they're written into the spec the compiler checks against.
- **Behaviours.** `use:name={params}` on an element can be backed by Rust instead of JavaScript: the backend
  hands the element's node and the params to an app-registered `Behavior`, which can attach native logic
  without a tree node of its own.
- **States.** An app can name its own state bits, so your stylesheets can match `:state(…)` selectors that
  Rust sets.

cadatomic, the CAD application skelter was built for, uses all three. Its Rust shell owns the window,
docking and tabs. It hands its own document to the backend (`SkelterBackend::shared`), so the Svelte panels
are nodes of the shell's tree rather than separate windows. It registers its own kinds, such as a property
grid and a code editor. It routes `use:command`, `use:op`, `use:live` and `use:presents` to Rust behaviours,
and it exposes states like `:state(cad.pending)` and `:state(cad.failed)` to the panels' CSS.

## When something goes wrong

Each layer fails in its own voice, and the first line of output usually tells you which layer it was.

- **The platform** speaks first. `neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set` (exit 1)
  means there's no display. A panic saying `libxkbcommon-x11.so could not be loaded` means a library winit
  loads at run time is missing. The binary links none of the display libraries; winit opens them when it
  starts, so these are never build errors.
- **The compositor** comes next. `svelters-skelter: cannot open window #1: Failed to create surface for any
  enabled backend` means no usable Vulkan or GL driver. The process then exits at once with status 0,
  because no window is left open. A window that opens but stays black, with no text at all, was seen with
  Mesa's software GL under Xvfb, and it prints nothing because nobody installed a logger. Your own binary can
  install one; see below.
- **The loader and compiler** speak in `Load: Error loading module` lines. An `error[code] (skelter)` after
  it is a compile error in your file; run `svelters-compile` on the file to see it in full. `… is not
  installed (pnpm install)` means the loader couldn't find the runtime or Svelte's files.
- **Your JavaScript** reports errors the host catches as `[svelters] …` lines, and your own logging as
  `[log] …`.

The compositor logs through Rust's `log` crate, which is already in the workspace's lockfile. To hear it, add
`log = "0.4"` to your crate's dependencies and a minimal logger to `main.rs`:

```rust
struct StderrLog;

impl log::Log for StderrLog {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Warn || m.target().starts_with("skelter")
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            eprintln!("[{}] {}: {}", r.level(), r.target(), r.args());
        }
    }
    fn flush(&self) {}
}

static LOG: StderrLog = StderrLog;
```

Install it as the first line of `main`:

```rust
    let _ = log::set_logger(&LOG).map(|()| log::set_max_level(log::LevelFilter::Info));
```

When the compositor creates its GPU device, it now prints which adapter and graphics API it chose, in a
line like `[INFO] skelter_compositor::gpu: skelter-compositor: llvmpipe (…) on Vulkan (Cpu)`. A failure
to find an adapter or to configure the window's surface shows up as an `[ERROR]` line instead of a silent
black window. skelter tries Vulkan before GL on Linux. In the runs behind this page, Vulkan drew (on the
software lavapipe driver) and GL on Mesa's software driver did not, so if you see GL in that line, install
a Vulkan driver.

## What isn't there yet

- **No release.** The crates are `publish = false` and the runtime package is private, so you work inside
  the svelters checkout, as this page does.
- **Linux only, as far as this page goes.** skelter's crates build and test on Linux, Windows and macOS in
  CI, but these steps have been run only on Linux with X11.
- **No window widget.** `window` is a placeholder kind, which is why it doesn't paint. The `text-edit`
  widget takes typing and updates `bind:text`, but doesn't draw itself yet.
- **No hot reload in the runner.** The loader and host have the pieces (an HMR compile mode, a reloader),
  but the runner doesn't watch your files, so every edit is a restart.
- **Templates aren't native yet.** The compiler emits them for skelter; the backend doesn't build them in
  one call yet, so the runtime expands them.
