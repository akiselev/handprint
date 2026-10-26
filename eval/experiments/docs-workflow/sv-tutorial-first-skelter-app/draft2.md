# Your first skelter app: a Svelte counter in a native window

This tutorial takes you from an empty directory to a native desktop window whose contents are a Svelte 5
component. You will write the component, open it with the runner that ships with svelters, make a button
change state, and then replace the runner with a Rust host of your own. The `.svelte` files are compiled
when the app starts, inside the Rust process, rather than bundled ahead of time.

Three pieces do the work:

- **skelter** is svelters' retained GUI framework. It owns the window, the widget tree, layout, text and
  a GPU compositor.
- **svelters-skelter** is the backend that lets compiled Svelte code create and update skelter widgets.
- **svelters-host** runs the compiled JavaScript in an embedded QuickJS engine, and its module loader
  compiles `.svelte` files with the svelters compiler as they are imported.

All three are unpublished. The crates are `publish = false` and the JavaScript runtime package is
private, so you work inside a checkout of the svelters repository.

## What you need

- A checkout of `https://github.com/akiselev/svelters`.
- Rust 1.97.0. The repository's `rust-toolchain.toml` pins it, so `rustup` picks it up when you run
  `cargo` in the checkout.
- Node.js 22 or newer and pnpm 10 (the root `package.json` names `pnpm@10.33.0`). Node only installs
  packages; the app itself runs in the Rust process.
- A C compiler. QuickJS is built from C sources by `rquickjs-sys`.
- On Linux: an X11 or Wayland session, libxkbcommon (plus libxkbcommon-x11 on X11), and a Vulkan driver.
  On Debian or Ubuntu that is `sudo apt install libxkbcommon-x11-0 mesa-vulkan-drivers`; the Mesa package
  includes lavapipe, a software Vulkan driver that works when there is no GPU.

winit, the windowing library, loads the display and GPU libraries when the program starts, so the binary
links none of them and a missing library shows up only at run time. winit uses Wayland when
`WAYLAND_DISPLAY` is set and X11 when only `DISPLAY` is.

skelter draws with Vulkan on Linux, falling back to GL; with Metal on macOS; and with DX12 on Windows. The
skelter crates are built and tested on all three in the repository's CI, but the steps on this page have
been run end to end only on Linux with X11.

The first build compiles a few hundred crates. On four cores with debug info turned off
(`CARGO_PROFILE_DEV_DEBUG=0`) it took under three minutes and about 2.4 GB of disk.

## Step 1: install the JavaScript packages

From the root of the checkout:

```sh
pnpm install
```

It takes a few seconds. pnpm warns in a box that it ignored the build script of `esbuild`; you can ignore
that warning for this tutorial.

The compiled component imports `@svelters/native-runtime`, and that runtime in turn loads parts of
`svelte` 5.57.2 from its `node_modules`. The module loader finds the runtime in the repository's
`packages/native-runtime` directory, but it can only find Svelte after `pnpm install` has run. Your app
directory can do without a `node_modules` of its own.

## Step 2: write a component

Create a directory for the app. This tutorial uses `examples/hello-skelter/app`; any directory works, as long
as it has a `package.json`.

```sh
mkdir -p examples/hello-skelter/app/src
```

Give it a `package.json`. The `svelters` object tells svelters' tools which toolkit the components are
written for:

```json
{
  "name": "hello-skelter",
  "private": true,
  "type": "module",
  "svelters": { "toolkit": "skelter" }
}
```

Then write `examples/hello-skelter/app/src/App.svelte`:

```svelte
<window title="Hello skelter" default-width={360} default-height={200}>
  <box gap={8}>
    <label>Hello from Svelte</label>
  </box>
</window>
```

None of these tags are HTML. Each one names a skelter widget kind from the skelter semantic spec
(`metadata/semantics/skelter.json`), and the compiler checks every tag, attribute and event against that spec:

- `<window>` is a native window. `title`, `default-width` and `default-height` set its title and its initial
  size in points (the defaults are 480 by 360). Its children are laid out as a flex column.
- `<box>` is a flexbox container that draws nothing itself. `gap` is the space between its children;
  `flex-direction` defaults to `column`.
- `<label>` is static text. Text written inside it becomes its `text` property.

The kinds you are most likely to reach for next are `button`, `checkbox`, `slider`, `text-edit`, `combo-box`
and `scroll-area`; the `tags` table in `skelter.json` lists all of them.

## Step 3: run it

From the root of the checkout, start the runner and point it at the app directory:

```sh
cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry examples/hello-skelter/app
```

`--entry` takes an app directory or a single file. For a directory, the loader mounts the `src/App.svelte`
inside it; the nearest `package.json`, in that directory or above it, marks the app root. If you give it a
`.svelte` file instead, that file is mounted.

When the build finishes, a 360 by 200 window titled "Hello skelter" opens. Its background is black, and
"Hello from Svelte" is drawn in light grey against the top-left corner. The black is the default theme, not
a rendering failure. Close the window and the process exits with status 0: the runner quits once the last
window it opened is gone.

Here is what happened. The runner made a skelter backend with the built-in widget kinds and the system
fonts, started the JavaScript host on it, and loaded your app through a source provider. The provider
generated a small entry module equivalent to `mount(App, { props: {} })`, compiled `App.svelte` for skelter,
and evaluated it. Mounting the component created a `window` node and called its `present` method. After that
call returned, the runner asked the platform for a window with the node's title and size, and attached the
node as the window's root.

If it fails to start, the terminal shows one of these messages:

- A panic saying `Library libxkbcommon-x11.so could not be loaded`: install libxkbcommon-x11 (on Debian or
  Ubuntu, `libxkbcommon-x11-0`).
- An error that starts with `svelters-skelter-run: event loop: os error at` and ends with `neither
  WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.`, exit status 1: there is no display. Run from a desktop session, or under Xvfb.
- `svelters-skelter: cannot open window #1: Failed to create surface for any enabled backend: {}`: there is no
  usable Vulkan or GL driver. The process exits at once with status 0, because no window is left open. Install
  a Vulkan driver (`mesa-vulkan-drivers`).
- An error ending in `is not installed (pnpm install)`, exit status 1: the module loader could not find Svelte
  or the native runtime. Run `pnpm install` in the checkout root (Step 1) and try again.
- `Load: Error loading module '<path>': <path>:2:4: error[unknown_element] (skelter): <push-button> is not an
  element of the skelter backend`, exit status 1: a compile error. The tag is not a skelter kind; a `help:`
  line suggests the nearest ones (here `button`, `radio-button`, `toggle-button`). Check the spelling against
  the `tags` table in `skelter.json`; GTK or Qt names such as `<push-button>` do not exist on skelter.

The load error is cut off after about 256 characters, and it names your file's path twice. In a deeply
nested checkout you may see only the path and lose the error code. Move the checkout or the app to a shorter
path to read the whole message.

If the window opens but stays black and the label never appears, skelter has probably fallen back to GL.
With Mesa's software GL driver under Xvfb, the window stays black and the terminal stays silent. Install a
Vulkan driver.

## Step 4: change state from a click

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

Run the same command again. The runner does not watch your files, so every edit needs a restart, and
each start compiles the app again. That is quick for a component this size.

The window now shows "Count: 0", "Doubled: 0" and an **Increment** button that spans the window's width.
Click **Increment** twice and the labels read "Count: 2" and "Doubled: 4". The terminal shows one line as
the app starts, before the window opens, and one per click:

```text
[log] count is 0
[log] count is 1
[log] count is 2
```

Three things to notice:

- `onclicked` listens to the button's native `clicked` signal ("pressed and released" in the spec). Handlers
  are `on` followed by the signal name. If you write a signal the element does not have, the compiler stops
  with `unknown_event`.
- `$state`, `$derived` and `$effect` are ordinary Svelte 5 runes. When `count` changes, only the labels that
  read it receive a new `text` value; skelter then lays out and draws the next frame.
- `console.log` goes to the terminal. The host echoes console output to stderr as `[level] text`, and
  JavaScript errors with the prefix `[svelters]`.

Pass `--dev` after the entry to compile with Svelte's `dev` option and the `development` export condition,
which gives you Svelte's development checks while you work.

## Step 5: host it from your own Rust binary

The runner is about seventy lines of Rust, and your own host can be the same code. Make
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

The workspace lists its members by name, so add the crate to `members` in the root `Cargo.toml`:

```toml
members = [
    # ...
    "examples/egui-kinds",
    "examples/hello-skelter",
]
```

Now write `examples/hello-skelter/src/main.rs`. It is the runner's `main` with the argument parsing removed
and the app directory fixed to the `app` folder next to the crate:

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

Run it:

```sh
cargo run -p hello-skelter
```

Cargo adds the new package to `Cargo.lock`. You get the same counter window and the same log lines as in
Step 4, now from your own binary.

Reading the host from top to bottom:

1. **Text.** One `FontStore` is shared by the widget tree (measuring), the widget kinds (shaping) and the
   compositor (rasterising). `FontConfig::default()` uses the system fonts. On a machine without fonts,
   `FontConfig::testing()` uses the Noto subsets bundled with skelter-text.
2. **Backend.** `SkelterBackend::with_text` creates a skelter document with the built-in kinds, and
   `install_widget_services` gives those widgets text shaping, the platform's interaction policy and the
   default theme.
3. **JavaScript host.** `Embed::with_backend` starts QuickJS on that backend. `HostConfig::default()` echoes
   console output and errors, which is where Step 4's log lines came from.
4. **Your app.** `SourceConfig::for_path` reads the app's `package.json` and picks `src/App.svelte` as the
   entry. Setting `cfg.backend = "skelter"` makes sure the components are compiled for skelter whatever the
   `toolkit` field says. `SourceProvider::new` locates the native runtime, and `load_app_from` compiles and
   evaluates the app, which mounts your component and presents its window.
5. **Loop.** `skelter::compositor` sets up the GPU compositor. The `Span` is the refresh interval new windows
   use until the platform reports the display's own: 16 667 µs is 60 Hz. `skelter_platform::run` runs the platform event loop on the main thread with `SveltersApp` as its handler.
   That handler opens a window for each `<window>` your code presents and exits when the last one closes.

The `svelters_host::modules` path is the svelters-modules crate. Its source provider sits behind the `source`
feature, which svelters-skelter already turns on, so your crate gets it without naming svelters-modules itself.

## What is missing so far

- **No release.** Nothing is on crates.io or npm, so a project outside the svelters checkout cannot depend on
  these crates in the usual way. Work inside the checkout as this tutorial does.
- **No automated test of the windowed runner.** The repository's tests drive the same pieces in skelter's
  headless test harness, but nothing runs `svelters-skelter-run` against a real window.
- **The docs site's getting-started guide is about GTK and Qt.** It uses the `counter` example binary, which
  does not run on skelter.

## Where to go next

- `fixtures/counter-egui/src/App.svelte` uses the same element vocabulary with more widgets: a `text-edit`
  with `bind:text`, a `slider` with `bind:value`, a `checkbox` with `bind:checked`, and a `<style>` block.
  Scoped `<style>` rules are applied from source on skelter, because skelter runs svelters' Rust CSS engine in
  process. Run it with `--entry fixtures/counter-egui`, or copy pieces of it into your `App.svelte`. The text
  field accepts typing and updates its bound value, but draws no box, text or caret yet.
- To put several Svelte components inside a window your Rust code owns, rather than letting the app open its
  own windows, use `Embed`'s slots: `Embed::adopt_slot` turns a node of your skelter document into a slot, and
  `Embed::mount` mounts a component into it. `a_svelte_root_mounts_into_a_node_of_the_host_document` in
  `crates/svelters-skelter/tests/host_document.rs` shows the whole sequence.
- To test a window without a display, look at `crates/svelters-skelter/tests/run.rs`, which drives a
  `skelter_test::Harness` and clicks a button by its label.
