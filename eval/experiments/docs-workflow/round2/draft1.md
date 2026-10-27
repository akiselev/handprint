# A Svelte counter in a native window, from the inside

In this tutorial you build a small app: a Svelte 5 counter in a native desktop window. First you run it
with the runner that ships with svelters, then from a Rust binary of your own. The app is only the vehicle.
What you are really here for is what happens underneath: how a `.svelte` file turns into nodes of a Rust
tree, how a click gets from the operating system to your handler, how `count++` turns into new pixels, and
where the Svelte side ends and the Rust side begins. At several points you'll stop and change something on
purpose to watch that machinery work.

Everything here runs inside a checkout of the svelters repository. Nothing is published to crates.io or npm
yet. The steps have been run end to end only on Linux, under X11 with Mesa's software Vulkan driver. They
haven't been tried on Wayland or on a real GPU.

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
  `sudo apt install libxkbcommon-x11-0 mesa-vulkan-drivers` covers the last two. The Mesa package includes
  lavapipe, a software Vulkan driver that works without a GPU, and it is the driver this page was run on.
  winit supports Wayland as well, but these steps weren't tried there.

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

The first build compiles a few hundred crates. On four cores with debug info off
(`CARGO_PROFILE_DEV_DEBUG=0`) it took between two and three minutes. Then a 360 by 200 window titled "Hello
skelter" opens. It is black, with "Hello from Svelte" in light grey in the top-left corner. The black is not
a rendering failure. Later on you'll change one line and watch it turn another colour. Close the window and
the process exits.

Here is what happened between `cargo run` and those pixels, in order:

1. The runner set up the tree. Its `main` made one font store, then a `SkelterBackend`. The backend owns
   an empty skelter-core `Document`, which is the tree, and knows the built-in kinds. It then installed what
   the kinds need: text shaping, the operating system's interaction rules, and a theme.
2. It started QuickJS on the main thread, attached to that backend. From now on, every host op your
   JavaScript makes is a direct Rust function call into the backend, which edits the `Document`. Values are
   converted on the way across, but nothing is serialised or queued in between.
3. It loaded your app from source. It read your `package.json` to find the app root and picked
   `src/App.svelte` as the entry. Then it evaluated a small generated entry module equivalent to
   `import App from './src/App.svelte'; mount(App, { props: {} })`. When QuickJS reached that import, it asked
   the module loader for the file, and the loader ran the svelters compiler on it right then, in Rust, and
   handed back JavaScript.
4. Mounting built the tree. The compiled component called the native runtime, and the runtime made host
   ops. It created a `window` node, a `box` and a `label`, set the window's `title`, `default-width` and
   `default-height` and the label's `text`, and inserted the label into the box and the box into the window.
   Finally it called the window's `present` method. At this point no operating-system window exists yet: the
   whole tree sits in the `Document`, detached.
5. The event loop started. The runner handed control to winit. Its app handler saw the presented window,
   read `title`, `default-width` and `default-height` back out of the node's props, and asked the platform for
   a window of that size. When the platform reported the window's first size, the handler made your `window`
   node that window's root.
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

Run the same command. No window opens, and the process exits with status 1. How much of the message you get
depends on how long your checkout's path is, for a reason you'll see in a moment. Here it is from a copy of
the app at the short path `/tmp/hs2/app`, where all of it fits:

```text
Load: Error loading module '/tmp/hs2/app/src/App.svelte': /tmp/hs2/app/src/App.svelte:3:6: error[unknown_element] (skelter): <push-button> is not an element of the skelter backend
  help: did you mean one of `button`, `radio-button`, `toggle-button`?
```

Read it from the left. `Load:` is svelters-host saying that loading your app failed. `Error loading module
'…'` is the JavaScript engine's binding reporting that it asked the loader for your module and got an error
back instead of code. Everything after the second path is the compiler's diagnostic. It gives the file, then
line 3, column 6, where the tag name starts just after the `<`. Then come the code `unknown_element`, the
backend it compiled for, and a `help:` line with the nearest names it knows.

How does the compiler know every tag skelter has? It compiles against a spec,
`metadata/semantics/skelter.json`, which is generated from the kinds registered in Rust. The spec lists every
tag, every prop with its type, and every signal. So the mistake is caught when the file is compiled, at
startup, with a suggestion, instead of turning into a node that silently draws nothing. The same spec catches
an event the element doesn't have. Change the tag back to `label` and add `onfoo={() => {}}` to it, and the
diagnostic becomes `…App.svelte:3:12: error[unknown_event] (skelter): <label> has no event `foo` on skelter`.

Now look at what your own checkout printed. On the machine behind this page the checkout sat deep in a
temporary directory, and all that came out was `Load: Error loading module '<path>': <path>`, cut off partway
through the second copy of the path, with no code and no message. QuickJS formats every error message it
throws into a fixed 256-byte buffer, so the text stops after 255 bytes. Your path appears twice, and the longer
it is, the less of the diagnostic survives. With a checkout at `/home/you/svelters`, the cut falls inside the
`help:` line.

The compiler is also a command-line tool, and it prints the whole diagnostic, with the path as you typed it.
The first run builds the CLI, which takes under half a minute:

```sh
cargo run -p svelters-compiler --bin svelters-compile -- --backend skelter examples/hello-skelter/app/src/App.svelte
```

```text
examples/hello-skelter/app/src/App.svelte:3:6: error[unknown_element] (skelter): <push-button> is not an element of the skelter backend
  help: did you mean one of `button`, `radio-button`, `toggle-button`?
```

It exits with status 1, as the runner did. Put the `<label>` back before you go on.

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
the app again. For a component this size you won't notice. The window shows "Count: 0", "Doubled: 0" and an
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

1. The platform reports a pointer press and a release. winit delivers them to the main thread, and each
   goes into skelter-core's dispatch as one transaction.
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
the component. It marks the signals that depend on `count` and schedules the effects that read them. The
labels' text is written by one template effect, which re-runs and computes `"Count: 1"` and `"Doubled: 2"`.
The runtime compares each value with the last one it wrote to that node, and only the changed ones cross into
Rust, as `setProp` calls on the two labels' handles.

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
Look closely after the first click and its fill is a shade lighter.

You can't watch the frame side directly from your app. Nothing on this page shows which display lists a
commit carried. For that half, the first lines of skelter-core's `frame.rs` and skelter-base's `commit.rs` are
the source. The JavaScript side you can count, and you will, a few sections from now.

## Experiment: read what the compiler made

You've seen the compiler's error output. Now look at its normal output for the counter:

```sh
cargo run -p svelters-compiler --bin svelters-compile -- --backend skelter examples/hello-skelter/app/src/App.svelte > /tmp/App.js
```

The JavaScript goes to stdout and Cargo's own lines go to stderr, so `/tmp/App.js` holds only the compiled
component. Here it is, all of it:

```js
import * as $ from '@svelters/native-runtime/core';
import * as $n from '@svelters/native-runtime';

export default function App($$anchor, $$props) {
	$.push($$props, true);
	let count = $.state(0);
	let doubled = $.derived(() => ($.get(count) * 2));
	$.user_effect(() => {
    console.log('count is', $.get(count));
  });
	const $$n1 = $n.toplevel("window", "present");
	$n.set($$n1, "title", "Hello skelter");
	$n.set($$n1, "default-width", 360);
	$n.set($$n1, "default-height", 200);
	const $$t0 = $n.instantiate($$T0, $n.slot($$n1, null), [() => 8]);
	const $$n3 = $n.at($$t0, 1);
	const $$n4 = $n.at($$t0, 2);
	const $$n5 = $n.at($$t0, 3);
	$n.append($$anchor, $$n1);
	$.template_effect(() => {
		$n.set($$n3, "text", `Count: ${$.get(count) ?? ''}`);
		$n.set($$n4, "text", `Doubled: ${$.get(doubled) ?? ''}`);
	});
	$n.template_listen($$t0, [() => $.update(count)]);
	$.pop();
}

const $$T0 = $n.template("299f0516aa46760e", [["box", null, [["setv", "gap", 0]], [1, 2, 3], null], ["label", null, [], [], null], ["label", null, [], [], null], ["button", null, [["set", "text", "Increment"]], [], null]], [[3, "clicked", 0]]);
```

Walk through it from the top.

The two imports are the two halves of the runtime. `$` is the reactive core, Svelte 5's own signals and
effects behind a thin adapter. `$n` is the native part, the one that makes host ops.

`$.push` and `$.pop` open and close the component's context, as in Svelte's own output. Your script survives
almost as you wrote it, with the runes turned into calls: `$.state(0)`, `$.derived(…)`, `$.user_effect(…)`.
Notice that every read of `count` became `$.get(count)`. That call is the whole trick behind fine-grained
reactivity. When an effect or a derived runs, each `$.get` records that it read this signal, so Svelte knows
exactly which effects to run again when `count` changes. `count++` in your handler became `$.update(count)`,
the write that sets them off.

`$n.toplevel("window", "present")` creates the window node. `"present"` names the method the runtime calls on
it once the component has finished building. The three `$n.set` lines after it write the window's props, once.
They aren't inside an effect because nothing in them can change.

Next comes the template. `$$T0` at the bottom of the file is a table built once, when the module loads. Node 0
is a `box` whose children are nodes 1, 2 and 3. Nodes 1 and 2 are labels, node 3 is a button with the static
text "Increment", and the last list, `[[3, "clicked", 0]]`, says that node 3 listens to `clicked` with
handler 0. The compiler saw that this subtree's shape never changes, so it described it once, as data.
`$n.instantiate($$T0, $n.slot($$n1, null), [() => 8])` builds it into the window's default slot. The array
holds the values the compiler didn't fold into the table: `gap={8}` is an expression in braces, so the table
says `["setv", "gap", 0]`, "set `gap` from value 0". The three `$n.at` lines fetch nodes 1 to 3 so the code
can write to them, `$n.template_listen` supplies handler 0, `() => $.update(count)`, and `$n.append` adds the
window to what the mount owns.

Then the part that runs again. `$.template_effect(…)` holds every reactive write in the fragment, here the
two labels' text, and it is the one effect that re-runs on a click. Each `$n.set(node, "text", value)` goes
through the runtime, which compares the value with the last one it wrote to that node and prop, and calls the
host only if it differs.

What you won't find is HTML, a `render()` function, or a virtual node. The component is a function that makes
host calls once, plus effects that make more host calls when the signals they read change.

Two details of this output are less than the spec allows.

The first is how props are named. They travel by name, as the string `"text"`, and svelters-host resolves
that name to the prop's id on each call. The spec also gives every prop a stable numeric id. Search
`metadata/semantics/skelter.json` for `135398928` and you'll find it under `Label` → `props` → `text`, next to
`"change": "layout"`. The compiler has a mode that writes those numbers into the code in place of names, so
that a write skips the lookup, and each module checks once, at load, that it was compiled against the same
spec as the backend it runs on. That mode is built, and skelter's spec lists `ids` among its capabilities,
but it doesn't switch on for skelter in current builds, so you see names.

The second is the template. It is an optimisation the skelter spec advertises (its `templates` capability),
meant to let the backend build the whole subtree in one call. svelters-skelter doesn't build templates
natively yet, so today the runtime expands the table into one `create`, `setProp` and `insert` per node, the
same calls the compiler would have written without templates. The spec describes what the compiler may emit,
and the backend decides what it can do in one step.

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

Run it. Three lines appear before the window does:

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

The handler runs to the end first. `count++` only marks things dirty, and the second `console.log` reads the
new value directly from the signal. Svelte runs the effects afterwards, in the microtask drain of the same
checkpoint. The template effect goes first and writes the labels, then your `$effect` runs, after the tree has
been updated. It's the same order as at mount, and for the same reason: Svelte runs template effects before
user effects, so that your `$effect` always sees the tree it describes.

Now add a value the click doesn't change, and a second button that does change it. Add a `name` state, a
third label that reads it through `show`, and a **Rename** button after **Increment**:

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

Predict twice. Does a click on **Increment** log `template reads name`? And what does a click on **Rename**
log?

Here is a run with one click on each, Increment first. Startup now reads `name` too, between `doubled` and the
effect:

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
`effect:`. All three labels' writes still sit in one template effect, so that effect runs on either click.
But each function call in a template is compiled into a derived value of its own, which computes again only
when what it read has changed. On Increment the `name` derived is still clean, so `show('name', …)` isn't
called. On Rename it's the `count` and `doubled` deriveds that are clean. Your `$effect` read only `count`,
so it doesn't run at all.

One more look at the compiler. Run `svelters-compile` on this file and find the name label's code:
`() => show('name', $.get(name))`. Now delete the **Rename** button and compile again. The line becomes
`() => show('name', name)`, with no `$.get`. The compiler saw that nothing ever assigns `name`, so it didn't
make a signal at all. A `$state` that nothing writes is just a variable, and there's nothing for anyone to
track.

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

Read the code against the model. Each block is one of the pieces from the start of the page:

- The tree and its kinds. `SkelterBackend::with_text(builtin_kinds(), …)` makes the `Document` on the
  registry of built-in kinds. The text system it receives measures text for layout.
  `install_widget_services` gives the document what the kinds read while they work: text shaping, the
  interaction policy for the current OS (how a press or a drag behaves), and the theme. One `FontStore` is
  shared three ways: by the tree for measuring, by the kinds for shaping and by the compositor for
  rasterising glyphs, so all three agree on the fonts.
- QuickJS. `Embed::with_backend` starts the JavaScript host on the backend. `HostConfig::default()` is
  where console echo comes from, and it also sets a 10-second budget for a single entry into JavaScript, so
  an endless loop in a handler is interrupted instead of freezing the window forever.
- The loader and compiler. `SourceConfig::for_path` reads `package.json`. Setting `cfg.backend` to
  `"skelter"` compiles against skelter's spec whatever the `toolkit` field says. `load_app_from` compiles and
  evaluates the app, which mounts the component and presents its window, all before any window exists.
- The compositor. `skelter::compositor` prepares the compositor thread, which starts once the platform is
  up. The `Span` is the frame interval new windows assume until the platform reports the display's real one
  (16,667 µs is 60 Hz).
- The event loop. `skelter_platform::run` runs winit on this thread with `SveltersApp` as its handler.
  That handler opens a window for each `<window>` the app presents, and the loop ends when the last one
  closes.

## Experiment: look at the tree, and count the crossings

Because you own `main` now, you can look at the tree directly. For this experiment, put `App.svelte` back to
the plain counter from "Making it react". Then add this function to `main.rs`:

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
(`SkelterBackend` is a cheap, shared handle). Dump the tree once the app has loaded. And print the backend's
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
called. Below it is the retained tree as Rust sees it, and it exists before any window does. The handles count
up in creation order, and they are the numbers JavaScript holds. JavaScript never holds a pointer into the
tree, only these numbers, which the backend maps to nodes and never reuses. That is why a handle to a
destroyed node can't reach a new one by accident. `change=Some(Layout)` is the change class from earlier,
read from the kind's spec: writing any of these texts re-measures the node. To see which props only repaint,
search `skelter.json` for `"change": "paint"`.

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

The first half is what Rust sent to JavaScript, the arrow from the tree back to QuickJS. `posts` counts
events posted to JavaScript. `syncs` counts events that JavaScript must answer before the native default can
go on, because a handler may cancel them. The spec marks `clicked` as `"sync": false`, so it's posted instead.
`changes` counts change notices, and this app binds nothing. `frames` counts animation frames handed to
JavaScript, and nothing here animates. `checkpoints` you can ignore: the embed runs a host checkpoint after
every platform callback, so it grows with mouse movement and timers whatever your app does.

Before you compare the runs, predict. From the click path above:

- Two clicks should add 2 to `posts` (one `clicked` event per click) and 4 to `sets` (two label texts per
  click) compared with the first run.
- Hovering should add nothing to `posts` or `sets`. skelter-core and the button's kind track hover in Rust,
  and no listener asked for pointer events.

Here is what came out:

| run | changed from run 1 |
|---|---|
| 2. two clicks | `posts: 2`, `sets: 11`, `checkpoints: 27` |
| 3. pointer on and off five times | `checkpoints: 117`, nothing else |

Every other counter stayed where it was. The moves cost 109 more checkpoints, one per platform callback,
and not one event or prop write.

While you do run 3, watch the button. In current builds the hover colour doesn't follow the pointer: moving
onto the button and off it leaves its fill as it was. The lighter hover fill appears only after a click, and
then it stays after the pointer leaves. That looks like a repaint bug rather than a design. The moves do reach
skelter-core (a pointer-move listener on the button gets called), but the button isn't redrawn for a hover
change until a press comes along. It doesn't change what the counters say: hover is Rust's business, and
nothing crossed.

One more, to see the runtime's dedupe as a number. Add a third label with a plain `{name}`, and no Rename
button:

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
reached the host. That is what "fine-grained" means here: work happens only where a value changed, first in
JavaScript and then again in Rust, where `set_prop` returns at once if the stored value is unchanged. Take the
label out again before you go on.

## Why the window is black

You can now explain the black background. Three facts combine:

- Nothing paints the window. skelter-widgets has no widget for `window` yet. svelters-skelter fills the
  gap with a placeholder kind: its props are real tree members (the app handler read `title` and the default
  size from them), but it records nothing. It's also why the window's documented default padding of 8 doesn't
  show and your text sits flush in the corner: layout comes from CSS, and no rule gives the window padding.
- The compositor clears to transparent. Where nothing draws, what you see is the compositor's clear
  colour, and the default is transparent. The platform window is opaque, so transparent comes out black.
- The theme only supplies colours. The runner installs VS Code's Dark Modern theme. Kinds use its
  colours for their own parts: the label draws light grey text and the button its fill and border. A theme
  doesn't draw a backdrop by itself.

Each of the three can be checked in turn.

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
you'll see, then run. You get a grey panel as wide as the window, because a box stretches across its parent,
which is a column flexbox like every node. It is only as tall as its content, with the content inset 12
pixels, and black below it. The `styles` counter reads 8 now, one more for the sheet. Add `flex-grow: 1;` to
the rule and the panel fills the window, because the window root is laid out at the window's size and the box
is its only child.

Next, separate theme from paint. Delete the `<style>` block, so nothing paints a backdrop again. Add
`skelter-widgets = { workspace = true }` to your crate's dependencies, and replace the
`install_widget_services` line with the same installation done by hand, using `Theme::light_modern()`:

```rust
    let policy = *skelter_widgets::interact::Native::for_os(skelter_widgets::interact::Platform::Linux).policy();
    backend.with_doc(|d| skelter_widgets::host::install(d, skelter_text::TextSystem::new(fonts.clone()), policy, skelter_widgets::theme::Theme::light_modern()));
```

(`install_widget_services` picks the interaction rules for the OS it was built for. This snippet names Linux,
which is fine for this page.) The text turns dark, the button light, and the background stays black. Dark text
on black is hard to read, and that is the point: the theme changed the kinds' colours, and still nothing
painted the window.

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

Put the dark theme and the default clear colour back before you go on: restore the `install_widget_services`
line and the original `spawn` line.

## Where Svelte stops and Rust starts

By now the dividing line should be visible. Svelte decides what exists and what values props have: which
nodes, in what order, with what text, and what to do when a signal arrives. Rust kinds do the work that
happens every frame or every pointer move: layout, painting, hit testing, hover and press, focus, scrolling,
caret movement. You counted that split: five passes of the pointer over the button posted no event and wrote
no prop, and each click posted one event. The split is deliberate. Direct manipulation never waits on
JavaScript, so a busy handler can't make hover or scrolling stutter. Crossing the boundary costs a function
call through QuickJS, so the design keeps it to the moments where your app actually has to decide something: a
click, a changed value.

The tree also has one owner per value. Some props are owned by the kind while you interact with it, such as
a slider's value during a drag or a scroll offset. When the kind changes one that JavaScript observes, for
example through `bind:value`, the new value goes back to JavaScript as a change notice: the `changes`
counter, which stayed at 0 here because nothing was bound. And if JavaScript writes such a prop based on an
older event than the kind's last change, skelter-core refuses the write and the native value stands. Stale
JavaScript can't paint over a live drag.

When an app needs more than the built-in kinds, it extends the Rust side at three points:

- **Kinds.** An app registers its own Rust kinds alongside the built-in ones (`Kinds::with_app`). They become
  tags your components can use, and they're written into the spec the compiler checks against.
- **Behaviours.** `use:name={params}` on an element can be backed by Rust instead of JavaScript: the backend
  hands the element's node and the params to an app-registered `Behavior`, which can attach native logic
  without a tree node of its own.
- **States.** An app can name its own state bits, so your stylesheets can match `:state(…)` selectors that
  Rust sets, the same way the button's hover and press bits drive its look.

cadatomic, the CAD application skelter was built for, uses all three. Its Rust shell owns the window,
docking and tabs, and hands its own document to the backend (`SkelterBackend::shared`), so the Svelte panels
are nodes of the shell's tree. It registers kinds such as a property grid and a code editor, routes
`use:command` and `use:op` to Rust behaviours, and exposes states like `:state(cad.pending)` to the panels'
CSS.

## When something goes wrong

Each layer fails in its own voice. They also fail in a fixed order, because of the startup order you saw at
the beginning: the app loads and mounts before the event loop starts. So the loader's errors and your own
`[log]` lines come out first, then the platform's, then the compositor's.

- The loader and compiler speak in `Load: Error loading module` lines. An `error[code] (skelter)` after
  it is a compile error in your file. If it's cut short, run `svelters-compile` on the file to see it in full.
  A line ending `is not installed (pnpm install)` means the loader couldn't find the runtime or Svelte's
  files. With `node_modules` moved away, it named `package "svelte" of alias target
  "svelte/src/easing/index.js"`.
- Your JavaScript reports errors the host catches as `[svelters] …` lines, and your own logging as
  `[log] …`, at load and whenever a handler or effect runs.
- The platform comes next. To see it, hide the display from the runner:
  `env -u DISPLAY -u WAYLAND_DISPLAY cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry examples/hello-skelter/app`.
  Your first `[log]` line still prints, because the app loaded fine. Then comes `svelters-skelter-run: event
  loop: os error at …: neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.`, and the process exits
  with status 1. A missing library shows up here too: on a machine without libxkbcommon-x11, winit panicked
  with `Library libxkbcommon-x11.so could not be loaded.` Your binary links none of the display libraries
  (`ldd` lists only libc, libm and libgcc_s). winit opens them when it starts, so these are never build
  errors.
- The compositor comes last, once a window exists. On a machine with no Vulkan driver installed,
  the runner printed `svelters-skelter: cannot open window #1: Failed to create surface for any enabled
  backend: {}` and exited at once with status 0, because no window was left open. The quieter failure is a
  window that opens and stays black, with no text at all. That means the compositor has no working GPU path:
  either no adapter at all, or a GL driver that doesn't draw. The runner says nothing in either case, because
  nobody installed a logger. Your own binary can.

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

The filter passes every warning and error from any crate, plus skelter's own info lines. Here is a run with
one click, with the tree dump shortened:

```text
[log] count is 0
<window> handle 1
  …
[WARN] winit::platform_impl::linux::x11::xdisplay: error setting XSETTINGS; Xft options won't reload automatically
[WARN] winit::platform_impl::linux::x11::util::randr: XRandR reported that the display's 0mm in size, which is certifiably insane
[ERROR] wgpu_hal::gles::egl: EGL 'eglInitialize' code 0x3001: DRI2: failed to load driver
[ERROR] wgpu_hal::gles::egl: EGL 'eglInitialize' code 0x3001: DRI2: failed to load driver
[INFO] skelter_compositor::gpu: skelter-compositor: llvmpipe (LLVM 20.1.2, 256 bits) on Vulkan (Cpu)
[log] count is 1
Traffic { syncs: 0, posts: 1, changes: 0, checkpoints: 21, frames: 0, creates: 5, tree: 4, sets: 9, gets: 3, calls: 1, listens: 1, styles: 7 }
```

The order is the startup order again: your effect and the dump, then the platform's lines once winit opens
the display, then the compositor's once it makes its GPU device. The line to read is the `INFO` one: the
compositor chose the llvmpipe adapter (lavapipe's CPU device) on Vulkan. The lines above it look alarming and
aren't. The two `WARN`s are winit grumbling about the bare virtual X server this ran on, and you may not see
them on a desktop. The two `ERROR`s are wgpu setting up its GL backend alongside Vulkan, and failing, because
this machine has no working GL driver. skelter tries Vulkan before GL on Linux, and Vulkan worked, so GL was
never needed. If you'd rather not see other crates' noise, change the filter to
`m.target().starts_with("skelter")`. Both of the compositor's lines that matter come from
`skelter_compositor::gpu`.

Now break it. Point the Vulkan loader at a driver list that doesn't exist:

```sh
VK_ICD_FILENAMES=/nonexistent.json cargo run -p hello-skelter
```

On the machine behind this page, where GL doesn't work either, the window opened and stayed black with no
text, exactly the silent failure from the list above. With the logger, the Vulkan loader's complaints and the
same EGL errors are followed by one line that names the cause:

```text
[ERROR] skelter_compositor::gpu: skelter-compositor: no GPU adapter for backends [Backends(VULKAN), Backends(GL)]
```

What happens on a machine where GL does work wasn't tried for this page. The `INFO` line is where you'd find
out which API the compositor fell back to.

## What isn't there yet

- No release. The crates are `publish = false` and the runtime package is private, so you work inside
  the svelters checkout, as this page does.
- Linux and X11 only, as far as this page goes. skelter's crates build and test on Linux, Windows and
  macOS in CI, but these steps have been run only on Linux under X11, with a software Vulkan driver.
- No window widget. `window` is a placeholder kind, which is why it doesn't paint. The `text-edit`
  widget takes typing and updates `bind:text`, but doesn't draw itself yet.
- No hot reload in the runner. The loader and host have the pieces (an HMR compile mode, a reloader),
  but the runner doesn't watch your files, so every edit is a restart.
- Templates aren't native yet. The compiler emits them for skelter, but the backend doesn't build them in
  one call yet, so the runtime expands them.
