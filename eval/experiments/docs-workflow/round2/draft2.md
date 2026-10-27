# A Svelte counter in a native window, from the inside

In this tutorial you build a Svelte 5 counter in a native desktop window, first with the runner that ships
with svelters, then from a Rust binary of your own. The app is only the vehicle. What you are here for is what
happens underneath: how a `.svelte` file turns into nodes of a Rust tree, how a click gets from the operating
system to your handler, how `count++` turns into new pixels, and where the Svelte side ends and the Rust side
begins. At several points you'll stop and change something on purpose to watch that machinery work.

Everything here runs inside a checkout of the svelters repository. The steps have been run end to end only
on Linux, under X11 with Mesa's software Vulkan driver, not on Wayland or a real GPU.

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
 ┌────────────────────────────── main thread ──────────────────────────────┐
 │                                                                         │
 │  winit ──input──▶ skelter-core tree ◀──── host ops ──── QuickJS         │
 │  event loop       (nodes of kinds)  ──── events, ─────▶ your compiled   │
 │                         │           change notices    Svelte code       │
 │                         │                             + runtime         │
 └─────────────────────────┼───────────────────────────────────────────────┘
                           │ commit: what changed this frame
                           ▼
             compositor thread ── wgpu ──▶ pixels in the window
```

Keep the two arrows between the tree and QuickJS in mind. JavaScript reaches the tree only through **host
ops**: create a node, insert it under another, set a prop, listen to a signal. The tree reaches JavaScript
in two ways only. One is **events**, and only the ones JavaScript listens to. The other is **change
notices** for the props JavaScript observes, such as a slider's value under `bind:value`. The rest of this
page fills in what travels along each arrow, and why. You'll count the traffic on both arrows before the end.

## Setting up

You need:

- a checkout of `https://github.com/akiselev/svelters`
- Rust 1.97.0 (the checkout's `rust-toolchain.toml` pins it, so `rustup` installs it on first use)
- Node.js 22 or newer and pnpm 10, only to install packages
- a C compiler, because QuickJS is built from C sources
- on Linux, an X11 session, libxkbcommon and libxkbcommon-x11, and a Vulkan driver. On Debian or Ubuntu,
  `sudo apt install libxkbcommon-x11-0 mesa-vulkan-drivers` covers the last two, including lavapipe, the
  software Vulkan driver this page was run on.

From the root of the checkout, install the JavaScript packages:

```sh
pnpm install
```

It takes a few seconds and adds 366 packages. pnpm then prints a boxed warning that it ignored esbuild's
build script. Nothing here needs esbuild.

Why does a native app need `node_modules` at all? Because the reactivity your components use is Svelte's
own: the native runtime imports Svelte 5.57.2's signals and effects directly, and the module loader reads
those files out of `node_modules` when the app starts. Node itself never runs your app. It only installed the
files.

## A window from a component

Make a directory for the app. Any directory with a `package.json` works, and this page uses
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
(`CARGO_PROFILE_DEV_DEBUG=0`) it took two to three minutes. Then a 360 by 200 window titled "Hello skelter"
opens. It is black, with "Hello from Svelte" in light grey in the top-left corner. The black is not a
rendering failure, and later you'll change one line to turn it another colour. Close the window and the
process exits.

Here is what happened between `cargo run` and those pixels, in order:

1. The runner set up the tree. It made a `SkelterBackend`, which owns an empty skelter-core `Document` (the
   tree) and knows the built-in kinds, and installed what the kinds need: text shaping, the operating
   system's interaction rules, and a theme.
2. It started QuickJS on the main thread, attached to that backend. From now on, every host op your
   JavaScript makes is a direct Rust function call into the backend, which edits the `Document`. Values are
   converted on the way across, but nothing is serialised or queued in between.
3. It loaded your app from source. It read your `package.json`, picked `src/App.svelte` as the entry, and
   evaluated a small generated module equivalent to `import App from './src/App.svelte'; mount(App, { props:
   {} })`. When QuickJS reached that import, the module loader ran the svelters compiler on the file right
   then, in Rust, and handed back JavaScript.
4. Mounting built the tree. The compiled component called the native runtime, and the runtime made host
   ops: create a `window`, a `box` and a `label`, set their props, insert the label into the box and the box
   into the window, and call the window's `present` method. No operating-system window exists yet: the whole
   tree sits in the `Document`, detached.
5. The event loop started. The runner handed control to winit. Its app handler saw the presented window,
   read `title`, `default-width` and `default-height` back out of the node's props, asked the platform for a
   window of that size, and made your `window` node that window's root.
6. A frame ran. On the main thread, skelter-core computed styles, laid the tree out with taffy (a flexbox
   engine), asked each dirty node's kind to record a display list (a few rectangles and a text run), and built
   the hit list that input will use. It packed what changed into a **commit** and sent it to the compositor
   thread, which drew it with wgpu and presented it.

Steps 3 and 4 happen before step 5, and that order will matter more than once on this page: your component
has run, and the whole tree exists, before there is any window to show it in.

Two ideas from that list are the heart of skelter.

The first is **retained**. The tree you built in step 4 stays in memory for the life of the window. Nothing
rebuilds it each frame. A frame only re-records the nodes something has changed, and the commit carries only
those nodes' display lists. Immediate-mode toolkits such as egui work the other way: they run your UI code
and rebuild every visible widget on every frame. Before skelter, svelters drew this kind of app with egui,
and that per-frame cost is the main reason skelter exists.

The second is **kinds**. `<window>`, `<box>` and `<label>` are not HTML elements. Each tag names a kind
registered in Rust, whose spec lists its props (each with a type and a change class, which comes up again
below), its signals and its slots, and whose code measures, paints, hit-tests and handles input. `box`,
`label` and `button` come from the `skelter-widgets` crate. In a browser the DOM plays this role. Here there
is no DOM, and no virtual DOM either: Svelte never needed one, since it compiles your markup into direct
calls, and svelters points those calls at the Rust tree instead of at a document.

## Experiment: break it on purpose

Experiments you'll throw away go in a scratch copy of the app, so your real app only moves forward. Put it
at a short path (you'll see why in a moment):

```sh
mkdir -p /tmp/hs2 && cp -r examples/hello-skelter/app /tmp/hs2/
```

In `/tmp/hs2/app/src/App.svelte`, change the label to a tag that doesn't exist on skelter. `<push-button>` is
a widget name from GTK and Qt:

```svelte
<window title="Hello skelter" default-width={360} default-height={200}>
  <box gap={8}>
    <push-button>Hello from Svelte</push-button>
  </box>
</window>
```

Run the copy:

```sh
cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry /tmp/hs2/app
```

No window opens, and the process exits with status 1:

```text
Load: Error loading module '/tmp/hs2/app/src/App.svelte': /tmp/hs2/app/src/App.svelte:3:6: error[unknown_element] (skelter): <push-button> is not an element of the skelter backend
  help: did you mean one of `button`, `radio-button`, `toggle-button`?
```

Read it from the left. `Load:` is svelters-host saying that loading your app failed. `Error loading module
'…'` is the JavaScript engine's binding: it asked the loader for your module and got an error back instead of
code. Everything after the second path is the compiler's diagnostic: line 3, column 6, where the tag name
starts, then the code `unknown_element`, the backend it compiled for, and the nearest names it knows.

How does the compiler know every tag skelter has? It compiles against a spec,
`metadata/semantics/skelter.json`, generated from the kinds registered in Rust: every tag, every prop with its
type, every signal. So the mistake is caught at startup, with a suggestion, instead of turning into a node
that silently draws nothing. The same spec catches an event the element doesn't have. Change the tag back to
`label` and add `onfoo={() => {}}` to it, and the diagnostic becomes `…App.svelte:3:12: error[unknown_event]
(skelter): <label> has no event `foo` on skelter`.

Now the short path. QuickJS formats every error message it throws into a fixed 256-byte buffer, so the text
stops after 255 bytes, and your path appears in it twice. From a checkout deep in a temporary directory, all
that came out was `Load: Error loading module '<path>': <path>`, cut off partway through the second copy of
the path. If that happens to you, the compiler's command-line tool, which you'll meet in a moment, prints the
whole diagnostic.

## Making it react

Back in your real app, replace `examples/hello-skelter/app/src/App.svelte` with a counter:

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

Run it again (the runner doesn't watch your files, so every edit needs a restart). The window shows "Count:
0", "Doubled: 0" and an **Increment** button as wide as the window. Click it twice and the labels read "Count:
2" and "Doubled: 4". The terminal shows:

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

1. The platform reports a pointer press and a release. winit delivers each to skelter-core's dispatch on
   the main thread, as one transaction.
2. Hit testing. skelter-core finds the node under the pointer using the hit list built by the last
   frame, so a click lands on what that frame put on screen.
3. Native defaults. The button's kind handles the input in Rust. It keeps hover and press as state bits
   on its node, and its hover and pressed looks come from style rules on those bits, the way `:hover` works
   in CSS. On the release over the button, it emits its `clicked` signal.
4. Only listened signals become events. The compiled code registered a listener for `clicked` when it
   built the button, so skelter-core lists a JavaScript-facing event. `press`, `release` and every pointer
   move have no listener, so they never leave Rust.
5. Delivery at event time. svelters-skelter posts the event to the host and runs a host checkpoint
   before the input's handling ends. QuickJS calls the runtime's dispatch function, which finds your handler
   by the node's handle and calls it. `count++` runs.
6. The microtasks drain inside that same checkpoint. That is where Svelte runs the effects that read
   `count`.

Then the new number has to reach the screen. Svelte's reactivity is fine-grained: `count++` doesn't re-render
the component, it schedules the effects that read `count`. The labels' text is written by one template
effect, which re-runs and computes `"Count: 1"` and `"Doubled: 2"`. The runtime compares each value with the
last one it wrote to that node, and only changed ones cross into Rust, as `setProp` calls on the labels'
handles.

In skelter-core, `set_prop` stores the value and marks the work that the prop's **change class** calls for.
Every prop in the spec has one of four classes, ordered by cost:

- `property`: only the compositor needs to know, as with opacity, a transform or a scroll offset.
- `paint`: re-record this node's display list, as with colours and borders.
- `layout`: restyle the node, lay it out again if its size changed, then re-record it, as with text, fonts,
  padding and gaps.
- `structure`: relayout, re-record, and rebuild the hit list and the accessibility tree. That is the cost of
  creating, inserting, removing or reordering nodes, so no prop declares it.

A label's `text` is layout class, so the label is re-measured, laid out again only if its size changed, and
marked for repainting. At the next frame those two labels re-record their display lists, and the commit
carries their new lists. `count++` doesn't touch the button or the box. The button does repaint on a click,
but for its own reason: its press and hover state changed, and Rust restyled it before any JavaScript ran.

You can't watch the frame side from your app (skelter-core's `frame.rs` and skelter-base's `commit.rs` are
the source for it), but the JavaScript side you can count, and you will.

## What the compiler made

The compiler is also a command-line tool. Point it at the counter (the first run builds the CLI, which takes
under half a minute):

```sh
cargo run -p svelters-compiler --bin svelters-compile -- --backend skelter examples/hello-skelter/app/src/App.svelte > /tmp/App.js
```

The JavaScript goes to stdout and Cargo's own lines go to stderr, so `/tmp/App.js` holds only the compiled
component, 28 lines of it. Here are the ones that teach:

```js
	let count = $.state(0);
	let doubled = $.derived(() => ($.get(count) * 2));
	…
	const $$n1 = $n.toplevel("window", "present");
	$n.set($$n1, "title", "Hello skelter");
	$n.set($$n1, "default-width", 360);
	$n.set($$n1, "default-height", 200);
	…
	$.template_effect(() => {
		$n.set($$n3, "text", `Count: ${$.get(count) ?? ''}`);
		$n.set($$n4, "text", `Doubled: ${$.get(doubled) ?? ''}`);
	});
```

`$` is the reactive core, Svelte 5's own signals and effects behind a thin adapter. `$n` is the native part,
the one that makes host ops. Every read of `count` became `$.get(count)`, and that call is the whole trick
behind fine-grained reactivity: when an effect or a derived runs, each `$.get` records that it read this
signal, so Svelte knows exactly which effects to run again when `count` changes.

The window's three `$n.set` lines run once, outside any effect, because nothing in them can change. The
`$.template_effect` holds every reactive write, here the two labels' text, and is the one effect that re-runs
on a click. (The elided lines build the box, labels and button from a table the compiler wrote once.) There
is no HTML, no `render()` function and no virtual node: the component makes host calls once, plus effects
that make more when the signals they read change.

## Experiment: watch the order of events

Make the order of events visible in the scratch copy. Replace `/tmp/hs2/app/src/App.svelte` with a version
that logs every step:

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

Run it with `--entry /tmp/hs2/app`. Three lines appear before the window does:

```text
[log] template reads count 0
[log] template reads doubled 0
[log] effect: count is 0
```

That is mounting: the template effect wrote the labels' first text, then your `$effect` ran. Now click once:

```text
[log] handler: count was 0
[log] handler: count is now 1
[log] template reads count 1
[log] template reads doubled 2
[log] effect: count is 1
```

The handler runs to the end first: `count++` only marks things dirty, and the second `console.log` reads the
new value straight from the signal. The effects run afterwards, in the microtask drain of the same
checkpoint: the template effect first, then your `$effect`, after the tree has been updated. Mount has the
same order, for the same reason: Svelte runs template effects before user effects, so your `$effect` always
sees the tree it describes.

Now add a `name` state, a third label that reads it through `show`, and a **Rename** button that changes
it:

```svelte
<script>
  // ...as before, plus:
  let name = $state('Ada');
</script>

<!-- ...inside the box, after the second label: -->
    <label>Name: {show('name', name)}</label>
<!-- ...and after the Increment button: -->
    <button onclicked={() => name = 'Grace'}>Rename</button>
```

Before you run it, predict: all three labels are written by the one template effect, so does a click on
**Rename** log the `count` and `doubled` lines too? Here is a run with one click on each, Increment first.
Startup now reads `name` too, between `doubled` and the effect:

```text
[log] template reads count 0
[log] template reads doubled 0
[log] template reads name Ada
[log] effect: count is 0
[log] handler: count was 0
[log] handler: count is now 1
[log] template reads count 1
[log] template reads doubled 2
[log] effect: count is 1
[log] template reads name Grace
```

**Increment** logs no `name` line, and **Rename** logs exactly one line: no `count`, no `doubled`, and no
`effect:`. The template effect does run on either click. But each function call in a template is compiled
into a derived value of its own, which computes again only when what it read has changed. On Increment the
`name` derived is still clean, so `show('name', …)` isn't called. On Rename it's the `count` and `doubled`
deriveds that are clean. Your `$effect` read only `count`, so it doesn't run at all.

One more look at the compiler. Run `svelters-compile` on this file and find the name label's code:
`() => show('name', $.get(name))`. Now delete the **Rename** button and compile again. The line becomes
`() => show('name', name)`, with no `$.get`. The compiler saw that nothing ever assigns `name`, so it didn't
make a signal at all. A `$state` that nothing writes is just a variable.

That's the last use for the scratch copy. Your real app is still the plain counter.

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

Run it with `cargo run -p hello-skelter`. Cargo adds the new package to `Cargo.lock` on the way. You get the
same window and the same log lines, now from a binary you own.

Read the code against the model; each block is one of the pieces from the start of the page:

- The tree and its kinds. `SkelterBackend::with_text(builtin_kinds(), …)` makes the `Document`, and
  `install_widget_services` gives it text shaping, the interaction policy for the current OS, and the theme.
  One `FontStore` serves the tree, the kinds and the compositor, so all of them agree on the fonts.
- QuickJS. `Embed::with_backend` starts the JavaScript host. `HostConfig::default()` gives you console echo
  and a 10-second budget for a single entry into JavaScript, so an endless loop in a handler is interrupted
  instead of freezing the window.
- The loader and compiler. `SourceConfig::for_path` reads `package.json`, and `load_app_from` compiles,
  evaluates and mounts the app, all before any window exists.
- The compositor. `skelter::compositor` prepares its thread. The `Span` is the frame interval new windows
  assume until the platform reports the display's real one (16,667 µs is 60 Hz).
- The event loop. `skelter_platform::run` runs winit on this thread with `SveltersApp` as its handler, which
  opens a window for each `<window>` the app presents and ends the loop when the last one closes.

## Experiment: look at the tree, and count the crossings

Because you own `main` now, you can look at the tree directly. Add this function to `main.rs`. It reaches
into backend internals that aren't documented API, so copy it as it is:

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
(`SkelterBackend` is a cheap, shared handle), dump the tree once the app has loaded, and print the backend's
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

Run it. Before the window appears, stderr shows this:

```text
[log] count is 0
<window> handle 1
  <box> handle 2
    <label> handle 3 text=Some(Str("Count: 0")) change=Some(Layout)
    <label> handle 4 text=Some(Str("Doubled: 0")) change=Some(Layout)
    <button> handle 5 text=Some(Str("Increment")) change=Some(Layout)
```

The `[log]` line comes first because your `$effect` ran during `load_app_from`, before `dump` was even
called. Below it is the retained tree as Rust sees it, before any window exists. The handles count up in
creation order, and they are all JavaScript ever holds: no pointers, only numbers the backend maps to nodes
and never reuses, so a handle to a destroyed node can't reach a new one by accident. `change=Some(Layout)` is
the change class from earlier, read from the kind's spec. To see which props only repaint, search
`skelter.json` for `"change": "paint"`.

The counters are the other half. The backend counts every call across the boundary, in both directions. Run
the binary three times:

1. Close the window without touching it.
2. Click **Increment** twice, then close.
3. Move the pointer onto the button and off it a few times without clicking, then close.

The first run prints:

```text
Traffic { syncs: 0, posts: 0, changes: 0, checkpoints: 8, frames: 0, creates: 5, tree: 4, sets: 7, gets: 3, calls: 1, listens: 1, styles: 7 }
```

Read the second half first: it's what JavaScript asked of Rust, and you can match it against the compiled code.
`creates: 5` is the window, the box, two labels and the button. `tree: 4` is four inserts: the box into the
window and three children into the box. `sets: 7` is the window's three props, the box's `gap`, the button's
text and the two label texts. `calls: 1` is `present`. `listens: 1` is the `clicked` listener. `gets: 3` is
the app handler reading `title`, `default-width` and `default-height` back for the platform, in step 5.
`styles` counts style operations, and you'll see it move when you add a stylesheet later.

The first half is the arrow from the tree back to QuickJS. `posts` counts events posted to JavaScript.
`syncs` counts events JavaScript must answer before the native default can go on, because a handler may
cancel them; the spec marks `clicked` as `"sync": false`, so it's posted. `changes` counts change notices
(this app binds nothing), and `frames` animation frames (nothing animates). `checkpoints` grows with every
platform callback, mouse movement and timers included, whatever your app does.

The click path says what the other two runs should show. Two clicks should add 2 to `posts` (one `clicked`
event per click) and 4 to `sets` (two label texts per click). Hovering should add nothing to either, because
skelter-core and the button's kind track hover in Rust and no listener asked for pointer events. Here is what
came out:

| run | changed from run 1 |
|---|---|
| 2. two clicks | `posts: 2`, `sets: 11`, `checkpoints: 27` |
| 3. pointer on and off five times | `checkpoints: 117`, nothing else |

Every other counter stayed where it was. The moves cost 109 more checkpoints, one per platform callback,
and not one event or prop write. (In current builds the button's hover fill also doesn't follow the pointer, which
looks like a repaint bug; either way, nothing crossed.)

One more, to see the runtime's dedupe as a number. Add a third label with a plain `{name}` to your real app,
and no Rename button:

```svelte
<script>
  // ...as before, plus:
  let name = $state('Ada');
</script>

<!-- ...inside the box, after the second label: -->
    <label>Name: {name}</label>
```

At startup `creates`, `tree` and `sets` each go up by one, for the new label. Two clicks still add exactly 4
to `sets`. The template effect wrote `"Name: Ada"` again on each click, and the runtime dropped it before it
reached the host. Work happens only where a value changed, first in JavaScript and then again in Rust, where
`set_prop` returns at once if the stored value is unchanged.

Take that label out again before you go on.

## Why the window is black

You can now explain the black background. Three facts combine:

- Nothing paints the window. skelter-widgets has no widget for `window` yet, so svelters-skelter uses a
  placeholder kind: its props are real (the app handler read `title` and the size from them), but it records
  nothing. That's also why your text sits flush in the corner: the window's documented default padding of 8
  is never read, because layout comes from CSS and no rule gives the window padding.
- The compositor clears to transparent. Where nothing draws, what you see is the compositor's clear
  colour, and the default is transparent. The platform window is opaque, so transparent comes out black.
- The theme only supplies colours. The runner installs VS Code's Dark Modern theme. Kinds use its
  colours for their own parts: the label draws light grey text and the button its fill and border. A theme
  doesn't draw a backdrop by itself.

Each can be checked in turn, in your own binary.

Something that paints has to cover the window, and `box` paints its own CSS box. Change the box to
`<box class="panel" gap={8}>` and add:

```svelte
<style>
  .panel {
    background-color: #2b2b2b;
    padding: 12px;
  }
</style>
```

skelter-core runs svelters' Rust CSS engine over the tree itself, so `<style>` works without any build step.
You get a grey panel as wide as the window (a box stretches across its parent, a column flexbox like every
node) and only as tall as its content, with black below it. The `styles` counter reads 8 now, one more for the
sheet. Add `flex-grow: 1;` to the rule and the panel fills the window.

Next, the theme. Delete the `<style>` block, add `skelter-widgets = { workspace = true }` to your crate's
dependencies, and replace the `install_widget_services` line with the same installation done by hand, naming
Linux and `Theme::light_modern()`:

```rust
    let policy = *skelter_widgets::interact::Native::for_os(skelter_widgets::interact::Platform::Linux).policy();
    backend.with_doc(|d| skelter_widgets::host::install(d, skelter_text::TextSystem::new(fonts.clone()), policy, skelter_widgets::theme::Theme::light_modern()));
```

The text turns dark, the button light, and the background stays black. The theme changed the kinds' colours,
and still nothing painted the window.

Last, the clear colour itself. Before you change it, predict which parts of the window will change colour.
Replace the `spawn` line with this, which sets the compositor's clear colour to navy:

```rust
    let spawn = skelter::compositor(|clock| {
        let mut c = skelter_compositor::CompositorConfig::new(clock, skelter_base::time::Span::from_micros(16_667));
        c.clear = Some(skelter_base::color::Color::from_rgba8(0, 0, 128, 255));
        c
    }, glyphs);
```

The whole background turns navy, and the light button and dark text stay exactly as they were. The colour
you've been calling the window's background was never the window's: it's what the compositor wipes damaged
areas to before it draws the display lists. `clear` is an `Option`, and `None` means "don't wipe, draw over
the previous frame".

## Where Svelte stops and Rust starts

By now the dividing line should be visible. Svelte decides what exists and what values props have: which
nodes, in what order, with what text, and what to do when a signal arrives. Rust kinds do the work that
happens every frame or every pointer move: layout, painting, hit testing, hover and press, focus, scrolling,
caret movement. You counted that split: five passes of the pointer posted no event and wrote no prop, and
each click posted one. Direct manipulation never waits on JavaScript, so a busy handler can't make hover or
scrolling stutter, and the boundary is crossed only when your app has to decide something.

The tree also has one owner per value. Some props, such as a slider's value during a drag or a scroll
offset, are owned by the kind while you interact with it. When the kind changes one that JavaScript observes
through `bind:value`, the new value goes back as a change notice (the `changes` counter, 0 here because
nothing was bound). If JavaScript writes such a prop based on an older event than the kind's last change,
skelter-core refuses the write, so stale JavaScript can't paint over a live drag.

When an app needs more than the built-in kinds, it extends the Rust side at three points:

- **Kinds.** An app registers its own Rust kinds alongside the built-in ones (`Kinds::with_app`). They become
  tags your components can use, written into the spec the compiler checks against.
- **Behaviours.** `use:name={params}` on an element can be backed by an app-registered Rust `Behavior`,
  which attaches native logic to the element's node without a node of its own.
- **States.** An app can name its own state bits, so your stylesheets can match `:state(…)` selectors that
  Rust sets, the way the button's hover and press bits drive its look.

cadatomic, the CAD application skelter was built for, uses all three: kinds such as a property grid and a
code editor, `use:command` routed to a Rust behaviour, and states like `:state(cad.pending)`. Its Rust shell
also hands its own document to the backend (`SkelterBackend::shared`), so the Svelte panels are nodes of the
shell's tree.

## When something goes wrong

Each layer fails in its own voice, and in a fixed order, because the app loads and mounts before the event
loop starts:

- **Loader and compiler**, while the app loads: `Load: Error loading module '…'`, followed by
  `error[code] (skelter)` for a compile error in your file. If it's cut short, run `svelters-compile` on the
  file. A line ending `is not installed (pnpm install)` means the loader couldn't find the runtime or
  Svelte's files in `node_modules`.
- **Your JavaScript**, at load and whenever a handler or effect runs: `[svelters] …` for errors the host
  catches, `[log] …` for your own logging.
- **The platform**, when the event loop starts. Hide the display
  (`env -u DISPLAY -u WAYLAND_DISPLAY cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry examples/hello-skelter/app`)
  and your first `[log]` line still prints, then `svelters-skelter-run: event loop: os error at …: neither
  WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.`, with status 1. Without libxkbcommon-x11, winit
  panicked with `Library libxkbcommon-x11.so could not be loaded.` winit opens the display libraries when it
  starts (`ldd` on your binary lists only libc, libm and libgcc_s), so these are never build errors.
- **The compositor**, once a window exists. With no Vulkan driver installed, the runner printed
  `svelters-skelter: cannot open window #1: Failed to create surface for any enabled backend: {}` and exited
  with status 0, because no window was left open. The quieter failure is a window that stays black with no
  text: no working GPU path, either no adapter at all or a GL driver that doesn't draw. The runner installs no
  logger, so it says nothing. The compositor logs through Rust's `log` crate, and a host that installs a
  logger gets the cause in one line:
  `skelter-compositor: no GPU adapter for backends [Backends(VULKAN), Backends(GL)]`.

## What isn't there yet

- No release. The crates are `publish = false` and the runtime package is private, so you work inside
  the svelters checkout, as this page does.
- No window widget. `window` is a placeholder kind, which is why it doesn't paint. The `text-edit`
  widget takes typing and updates `bind:text`, but doesn't draw itself yet.
- No hot reload in the runner. The loader and host have the pieces (an HMR compile mode, a reloader),
  but the runner doesn't use them yet.
