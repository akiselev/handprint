# Your first skelter app: a Svelte counter in a native window

This tutorial takes you from an empty directory to a native desktop window whose contents are a Svelte 5
component. You will write the component, open it with the runner that ships with svelters, make a button
change state, and then replace the runner with a Rust host of your own. Nothing is bundled ahead of time:
the `.svelte` files are compiled when the app starts, inside the Rust process.

Three pieces do the work:

- **skelter** is svelters' retained GUI framework. It owns the window, the widget tree, layout, text and
  a GPU compositor.
- **svelters-skelter** is the backend that lets compiled Svelte code create and update skelter widgets.
- **svelters-host** runs the compiled JavaScript in an embedded QuickJS engine, and its module loader
  compiles `.svelte` files with the svelters compiler as they are imported.

There is no published release of any of these. The crates are `publish = false` and the JavaScript runtime
package is private, so you work inside a checkout of the svelters repository.

## What you need

- A checkout of `https://github.com/akiselev/svelters`.
- Rust 1.97.0. The repository's `rust-toolchain.toml` pins it, so `rustup` picks it up when you run
  `cargo` in the checkout.
- Node.js 22 or newer and pnpm 10 (the root `package.json` names `pnpm@10.33.0`). Node is only used to
  install packages; the app itself never runs in Node.
- A C compiler. QuickJS is built from C sources by `rquickjs-sys`.
- On Linux: an X11 or Wayland session, the Wayland client library (skelter's platform crate links it), and
  a Vulkan or OpenGL driver. skelter tries Vulkan first and then GL; a software driver such as lavapipe or
  llvmpipe works but is chosen last. On macOS the compositor uses Metal, on Windows DX12.

The first build compiles a large workspace. Give it time.

## Step 1: install the JavaScript packages

From the root of the checkout:

```sh
pnpm install
```

The compiled component imports `@svelters/native-runtime`, and that runtime in turn loads parts of
`svelte` 5.57.2 from its `node_modules`. The module loader finds the runtime in the repository's
`packages/native-runtime` directory, but it can only find Svelte after `pnpm install` has run. Your own app
directory does not need a `node_modules` of its own.

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

`--entry` takes an app directory or a single file. For a directory, the loader looks for the nearest
`package.json` and mounts `src/App.svelte`. If you give it a `.svelte` file instead, that file is mounted.

When the build finishes, a window titled "Hello skelter" opens with the label in it. Close the window and the
process exits: the runner quits once the last window it opened is gone.

Here is what happened. The runner made a skelter backend with the built-in widget kinds and the system
fonts, started the JavaScript host on it, and loaded your app through a source provider. The provider
generated a small entry module equivalent to `mount(App, { props: {} })`, compiled `App.svelte` for skelter,
and evaluated it. Mounting the component created a `window` node and called its `present` method. After that
call returned, the runner asked the platform for a window with the node's title and size, and attached the
node as the window's root.

**If it fails to start.** The runner prints the reason and exits with status 1. Two messages are common:

- An error ending in `is not installed (pnpm install)` means the module loader could not find Svelte or the
  native runtime. Run `pnpm install` in the checkout root (Step 1) and try again.
- A compile error names your file and the problem. The usual one on a first try is `unknown_element`: the tag
  is not a skelter kind. Check the spelling against the `tags` table in `skelter.json`; GTK or Qt names such as
  `<push-button>` do not exist on skelter.

If the window never appears and you see `svelters-skelter: cannot open window`, the platform could not create
a window, which usually means there is no display or no usable GPU driver.

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

Run the same command again. The runner does not watch your files, so every edit needs a restart; the
loader caches compiled output by content and only recompiles files that changed.

The window now has two labels and a button. Click **Increment** and both labels update. In the terminal you
should see one line when the window opens and one per click, like this:

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
  JavaScript errors as `[svelters] …`.

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

You get the same counter window as in Step 4, now from your own binary.

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
feature, which svelters-skelter already turns on, so your crate does not need to name svelters-modules itself.

## What is not there yet

- **No release.** Nothing is on crates.io or npm, so a project outside the svelters checkout cannot depend on
  these crates in the usual way. Work inside the checkout as this tutorial does.
- **No reload in the runner.** Edits to `.svelte` files need a restart.
- **The window path is new.** The repository's tests cover the parts separately: a window built by JavaScript
  opening in skelter's headless test harness with a click on its button reaching JavaScript
  (`crates/svelters-skelter/tests/run.rs`), and a component compiled from source whose `onclicked` handler
  updates a label (`crates/svelters-skelter/tests/host_document.rs`). No test in the repository yet opens a
  real platform window from `svelters-skelter-run --entry`.
- **The docs site's getting-started guide is about GTK and Qt.** It uses the `counter` example binary, which
  does not run on skelter.

## Where to go next

- `fixtures/counter-egui/src/App.svelte` uses the same element vocabulary with more widgets: a `text-edit`
  with `bind:text`, a `slider` with `bind:value`, a `checkbox` with `bind:checked`, and a `<style>` block.
  Scoped `<style>` rules are applied from source on skelter, because skelter runs svelters' Rust CSS engine in
  process. Copy pieces of it into your `App.svelte`.
- To put several Svelte components inside a window your Rust code owns, rather than letting the app open its
  own windows, use `Embed`'s slots: `Embed::adopt_slot` turns a node of your skelter document into a slot, and
  `Embed::mount` mounts a component into it. `a_svelte_root_mounts_into_a_node_of_the_host_document` in
  `crates/svelters-skelter/tests/host_document.rs` shows the whole sequence. cadatomic hosts its panels this
  way.
- To test a window without a display, look at `crates/svelters-skelter/tests/run.rs`, which drives a
  `skelter_test::Harness` and clicks a button by its label.
