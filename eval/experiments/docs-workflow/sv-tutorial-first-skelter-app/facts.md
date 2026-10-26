# Facts: sv-tutorial-first-skelter-app

svelters checkout at `6c1ea458` (/home/user/svelters). Paths are relative to it unless marked
`cadatomic:`. Nothing was built or run by the writer. Stage 3 (reviser) corrections are marked **[rev]**; "observed: verify.md
 execution log" means the verifier's real run under Xvfb (Ubuntu 24.04, Mesa 25.2.8, svelters `6c1ea458`).

## The runner binary

- F1. `svelters-skelter` has a binary `svelters-skelter-run` with two modes: `[--bundle] <dist/app.js>`
  (a bundle built for skelter) and `--entry <app dir | file> [--dev]` (from source, compiled live for
  skelter). — `crates/svelters-skelter/src/bin/svelters-skelter-run.rs:1-6`, `:16`
- F2. "The app mounts its `<window>`s itself; each one it presents opens a platform window (winit) drawn
  by skelter's compositor (wgpu). Text uses skelter-text over the system fonts." — `svelters-skelter-run.rs:8-9`
- F3. With `--entry`, the runner builds a `SourceConfig::for_path(&entry)`, sets `cfg.backend = "skelter"`
  and `cfg.dev = dev`, makes a `SourceProvider`, and calls `embed.load_app_from(Rc::new(provider))`.
  — `svelters-skelter-run.rs:56-62`
- F4. Load errors are printed to stderr and the process exits with status 1 (`fail`); no arguments
  prints the usage and exits 2. — `svelters-skelter-run.rs:39-46`, `:65-67`
- F5. Runner setup order: `FontStore::new(FontConfig::default())`; `SkelterBackend::with_text(builtin_kinds(), TextSystem)`;
  `install_widget_services`; `HostConfig::default()`; `Embed::with_backend(backend, EmbedConfig::app(host))`;
  load; `FontRasterizer`; `skelter::compositor(... CompositorConfig::new(clock, Span::from_micros(16_667)), glyphs)`;
  `skelter_platform::run(Box::new(SveltersApp::new(embed, spawn)))`. — `svelters-skelter-run.rs:48-73`
  (the main.rs code block in the draft is adapted from these lines: the argument parsing is replaced by a fixed
  `app` directory next to the crate's manifest).
- F6. The crate's dependency on svelters-modules turns on feature `source` for the `--entry` path.
  — `crates/svelters-skelter/Cargo.toml:21-22`
- F7. Dependency list of svelters-skelter (skelter-base, skelter-core, skelter-motion, skelter, skelter-platform,
  skelter-compositor, skelter-text, skelter-widgets, svelters-abi, svelters-host, svelters-modules, …).
  — `crates/svelters-skelter/Cargo.toml:9-24`

## Loading from source

- F8. `SourceConfig::for_path(path)`: the app at `path` is a directory (its embed entry, or `src/App.svelte`)
  or a file inside an app (`.svelte` mounts it, anything else is evaluated as is); the app root is the nearest
  directory with a package.json; error "no package.json in it or above it" otherwise.
  — `crates/svelters-modules/src/source/mod.rs:78-88`, `:94-108`
- F9. A directory without an `"embed"` table in package.json's `"svelters"` object gets entry
  `src/App.svelte`. — `mod.rs:94-103`
- F10. `"svelters": { "toolkit" }` is read as the backend; default `gtk`. — `mod.rs:92-93`, `:111`
- F11. The generated entry for an App is `import { mount } from '@svelters/native-runtime'; import App from …; mount(App, { props: {} });`.
  — `mod.rs:376`
- F12. `dev`: Svelte's `dev` option and the `development` export condition instead of `production`.
  — `mod.rs:65-66`, `:356-358`
- F13. The native runtime directory is looked up in the app's `node_modules`, through `@svelters/build`'s,
  or this repository's `packages/native-runtime` (via the svelters-modules crate's own manifest dir); else the
  error "@svelters/native-runtime is not installed (pnpm install)". — `mod.rs:131-143`
- F14. The runtime's alias table maps `svelte` and Svelte internals to files; a target in a package that is not
  installed fails with "package … of alias target … is not installed (pnpm install)". Package directories are
  found by walking `node_modules` up from the runtime directory. — `crates/svelters-modules/src/source/aliases.rs:22-45`, `:86-88`
- F15. `@svelters/native-runtime` depends on `svelte` 5.57.2 and is `"private": true` (not published).
  — `packages/native-runtime/package.json:3-5` and its `dependencies`; `packages/build/package.json:5` private too.
- F16. Compiled components import `@svelters/native-runtime/core` and `@svelters/native-runtime`.
  — `crates/svelters-compiler/src/target.rs:10-21`
- F17. A component with compile errors fails the load with the rendered errors joined by newlines (file path
  shown). — `crates/svelters-modules/src/source/transform.rs:113-118`
  **[rev]** Incomplete: the message reaches the user through a QuickJS module-load error, shaped
  `Load: Error loading module '<path>': <path>:L:C: error[code] (skelter): …` plus a `help:` line, and is cut at
  about 256 bytes; with a long path only the path shows. — observed: verify.md execution log, Step 3 (issue 2)
- F18. `<style>` is applied from source on backends that run the CSS engine (egui, skelter).
  — `transform.rs:120-124`
- F19. `SourceProvider` keeps transform output by content, so a reload recompiles only what changed.
  — `mod.rs:276-279`
  **[rev]** The cache is an in-memory map per `SourceProvider` (`mod.rs:266`, `:480-498`); the runner never
  reloads and opens no on-disk bytecode cache, so every start recompiles. Not usable for the draft's restart claim.
- F20. The runner does not watch files or reload: no watch/reload code in `bin/svelters-skelter-run.rs`,
  `app.rs` or `embed/`; `cfg.hmr` is never set by the runner (default `false`, `mod.rs:119`). The embed's
  "reload pending" state is reserved. — `crates/svelters-skelter/src/embed/settle.rs:37`

## Element vocabulary (skelter spec)

- F21. The skelter semantic spec is `metadata/semantics/skelter.json`, compiled into svelters-compiler.
  — `crates/svelters-compiler/src/semantics/backend.rs:15`, `:62`
- F22. Tags include `box`, `button`, `label`, `window`, `checkbox`, `slider`, `text-edit`, `combo-box`,
  `scroll-area`, `virtual-list`, … **[rev]** (34 tags; `kinds` has 37). — `metadata/semantics/skelter.json` `tags` (line ~24ff)
- F23. `Window`: props `title`, `default-width` (480), `default-height` (360), `gap`, `padding` (8); method
  `present`; "its children are laid out as a flex column"; `present: "present"`.
  — `skelter.json:9644`, `:9689`, `:9704`, `:9706`, `:9726`, `:9806`
- F24. `Button`: signal `clicked` ("Pressed and released"), `text_content: "text"` (text children become `text`).
  — `skelter.json:647`, `:860`, `:937`
- F25. `Label`: `text_content: "text"`; static text. — `skelter.json:2707`, `:3009`
- F26. `Box`: "A flexbox container laid out by taffy; draws nothing itself"; `flex-direction` (column, row,
  column-reverse, row-reverse; default column), `gap` (default 0). — `skelter.json:387`, `:472`, `:548`
- F27. Text alone in a text-content element folds into its text prop. — `crates/svelters-compiler/src/semantics/spec.rs:38-41`
- F28. Compiler error codes: `unknown_element` ("The tag is not an element of the backend's semantic spec"),
  `text_not_allowed`, `unknown_property`, `unknown_event`. — `crates/svelters-compiler/src/codes.rs:73`, `:75`, `:80`, `:84`
- F29. The runtime presents a toplevel through the spec's present method (falls back to `present`).
  — `packages/native-runtime/src/index.js:440`, `:468-476`, `:527-528`

## What happens at run time

- F30. `SveltersApp`: JS builds a `window` node and calls `present`; after the callback the app asks the platform
  for a window with the node's title and `default-width`/`default-height`; the window's first size attaches the
  JS-built node as its root. — `crates/svelters-skelter/src/app.rs:1-15`, **[rev]** `:178-187`
- F31. `window_desc` reads `title`, `default-width` (default 480) and `default-height` (default 360).
  — `crates/svelters-skelter/src/backend.rs:763-774`
- F32. The app exits when the backend is quitting or when it has opened a window and none remain.
  — **[rev]** `app.rs:209`; observed exit 0 on close (verify.md execution log, Step 3)
- F33. If the platform cannot open a window, the runner prints `svelters-skelter: cannot open window #<h>: <e>`.
  — **[rev]** `app.rs:184`. Observed with no GPU driver: `svelters-skelter: cannot open window #1: Failed to create
  surface for any enabled backend: {}`, then exit 0 at once (`opened_any` stays false, `app.rs:182-185`, `:209`).
  It is not the no-display message (F68).
- F34. `HostConfig::default()`: `echo_console: true`, `echo_errors: true`, `quit_on_last_window: true`,
  `turn_budget` 10 s, memory limit 256 MiB. — `crates/svelters-host/src/core.rs:95-114`
- F35. Console output is echoed to stderr as `[<level>] <text>`; `console.log` has level `log`; arguments are
  joined with spaces (strings as is, objects as JSON). — `crates/svelters-host/src/prelude.js:4-22`, `core.rs:626-630`
- F36. Reported JS errors are echoed as `[svelters] <err>`. — `core.rs:591-597` (`pub(crate) fn report`)
- F37. `EmbedConfig::app(host)` sets every budget to the host's turn budget (default a day if none).
  — `crates/svelters-skelter/src/embed/mod.rs:100-103`
- F38. `Embed::load_app_from`: "[`Embed::load_app`] from a module provider (an app compiled from source)";
  `load_app` evaluates a bundle that mounts its `App` at the root itself. — `embed/mod.rs:443-460`

## Platform and toolchain

- F39. `skelter_platform::run` runs a winit event loop on the main thread. — `crates/skelter-platform/src/backend.rs:44-58`
- F40. winit 0.30.13 with features `x11`, `wayland`, `wayland-dlopen`; wgpu 30.0.1. — `Cargo.toml:101-102`
- F41. ~~skelter-platform links `wayland-backend`~~ **[rev] doubtful, not used:** the Cargo line exists
  (`crates/skelter-platform/Cargo.toml:56`), but the built binary links only libc, libm, libgcc_s (`ldd`); winit's
  `wayland-dlopen` loads Wayland at run time. — observed: verify.md F41 row; `Cargo.toml:102`
- F42. GPU: Vulkan then GL on Linux, Metal on Apple, DX12 on Windows; software adapters (lavapipe, llvmpipe, WARP)
  ranked last. — `crates/skelter-compositor/src/gpu.rs:49-59`, `:85-90`
  **[rev]** Software is ranked last only within one backend: the first backend in order with any presentable
  adapter wins, so lavapipe (Vulkan) beats a hardware GL driver. — `gpu.rs:49-59`, `:85-90`
- F43. `FontConfig::default()` uses system fonts, default family `system-ui`; `FontConfig::testing()` uses bundled
  Noto subsets and no system fonts. — `crates/skelter-text/src/fonts.rs:41-56`
- F44. Rust toolchain pinned to 1.97.0. — `rust-toolchain.toml:2`
- F45. Node >= 22, pnpm 10.33.0. — `package.json:6-8`
- F46. QuickJS comes from `rquickjs` 0.14.0 (`rquickjs-sys` builds with `cc`, i.e. a C compiler).
  — `Cargo.toml:90`, `Cargo.lock:4491-4497`
- F47. Workspace members are listed explicitly (no glob); every crate is `publish = false`; licence MIT.
  — `Cargo.toml:3-48`, `:50-54`
- F48. Workspace path dependencies: `svelters-skelter = { path = "crates/svelters-skelter" }`, `skelter`, `skelter-text`, etc.
  — `Cargo.toml:57-73`
- F49. `tools/dev/headless` runs a command on a private Xvfb; `SVELTERS_SHOW_GUI=1` uses the real display.
  — `tools/dev/headless:1-8`

## Tests that cover the pieces

- F50. `tests/run.rs` `a_presented_window_opens_and_its_button_clicks_reach_js`: a JS-built window presented
  opens in skelter-test's `Harness`, and `h.get_by_label("Increment").click()` reaches JS. — `crates/svelters-skelter/tests/run.rs:11-36`
- F51. `tests/host_document.rs` `a_skelter_click_runs_the_svelte_onclicked_handler`: a component compiled from source
  for skelter (`<button text="Go" onclicked={() => count++} />`, `<label text={`n=${count}`} />`) goes from `n=0`
  to `n=1` after a `clicked`. Needs `pnpm install`. — **[rev]** `host_document.rs:152-200`, `:91` (injects
  `clicked` with `target_event`, not a pointer click)
- F52. The app's own milestone record lists running `svelters-skelter-run --entry fixtures/counter-egui` under
  `tools/dev/headless` "once a GPU/lavapipe is at hand" as next work, i.e. the `--entry` window path is not recorded
  as exercised. — `plans/milestones/SKELTER-svelters-skelter.md:185-186`
- F53. `fixtures/counter-egui/src/App.svelte` is a larger example in the same element vocabulary (`window` with
  `default-width`/`default-height`, `box`, `label`, `button onclicked`, `text-edit bind:text`, `slider bind:value`,
  `checkbox bind:checked`, `<style>`). — `fixtures/counter-egui/src/App.svelte:1-33`
- F54. Runes `$state`, `$derived`, `$effect` are used in components compiled for skelter / egui from source.
  — `crates/svelters-skelter/tests/app_values.rs:111-122` (`$state`, `$effect`), `fixtures/embed-egui/src/TaskPanel.svelte:15` (`$derived`)

## Other hosts

- F55. `Embed` mounts many Svelte roots into slots of a host's own skelter document (`SkelterBackend::over`/`shared`,
  `Embed::adopt_slot`, `MountSpec`). — `crates/svelters-skelter/tests/host_document.rs:1-3`, `:111-139`
- F56. cadatomic hosts its Svelte panels that way: `SkelterBackend::shared(...)` over the shell's document.
  — cadatomic: `crates/cadatomic-app/src/skelter/tree.rs:3`, `:67`; `crates/cadatomic-ui/src/app_embed.rs:1173`
- F57. The docs site's getting-started guide covers GTK and Qt (`counter` binary), not skelter.
  — `docs/site/src/content/docs/guides/getting-started.md:1-4`

## Code blocks in the draft and their sources

- C1 `App.svelte` (static): adapted from `fixtures/counter-egui/src/App.svelte:11-14` and
  `fixtures/counter/src/App.svelte`.
- C2 `package.json`: shape from `crates/svelters-skelter/tests/host_document.rs:96` and
  `fixtures/counter-egui/package.json` (`"svelters": { "toolkit": … }`), minus the embed table and scripts.
- C3 `App.svelte` (counter): `fixtures/counter-egui/src/App.svelte:5,11-14` plus `$derived`/`$effect` (F54);
  `<button onclicked={() => count++}>Increment</button>` is verbatim from the fixture.
- C4 run command: bin name from `crates/svelters-skelter/src/bin/svelters-skelter-run.rs` (file name = bin name), flags from its `USAGE` (`:16`).
- C5 `Cargo.toml` of the new crate: workspace inheritance as in `crates/svelters-skelter/Cargo.toml:1-24`.
- C6 `main.rs`: adapted from `svelters-skelter-run.rs:11-14`, `:43-73`.
- C7 the `members` edit: `Cargo.toml:3`, `:47`.

## Added while drafting

- F58. `install_widget_services` gives the backend's document what skelter-widgets' kinds read: `text` (shaping),
  the OS's platform policy and the built-in theme. — **[rev]** `crates/svelters-skelter/src/app.rs:35-47`
- F59. `SkelterBackend::with_text(kinds, text)` creates a fresh `Document` on the kinds' registry measuring with
  `text`; `SkelterBackend::new` measures no text. — `crates/svelters-skelter/src/backend.rs:292-301`
- F60. `CompositorConfig::new(clock, refresh)`: `refresh` is "the refresh interval of new windows until the
  platform reports theirs". — `crates/skelter-compositor/src/lib.rs:73-74`, `:92-104`
- F61. egui and skelter run the Rust CSS engine in process and get scoped CSS as Svelte wrote it.
  — **[rev]** `packages/native-runtime/vite.js:396` (also `crates/svelters-skelter/src/spec.rs:311`)
- F62. `SourceProvider::new` locates the runtime (`find_runtime`) and loads its alias table before anything
  compiles. — `crates/svelters-modules/src/source/mod.rs:283-286`
- F63. `Embed::adopt_slot(key, node)` and `Embed::mount(MountSpec::new(entry, export, slot, key))` mount a
  component into a node of the host's document. — `crates/svelters-skelter/tests/host_document.rs:118-123`
- C8 the `[log] count is …` text block. **[rev]** Now observed verbatim on stderr after two clicks, in Step 4 and
  Step 5. — observed: verify.md execution log, Steps 4 and 5

## Added in revision (stage 3)

- F64. On X11 the first run panicked before any window: `Library libxkbcommon-x11.so could not be loaded`
  (from `xkbcommon-dl-0.4.2/src/x11.rs:59`); installing `libxkbcommon-x11-0` fixed it. — observed: verify.md
  execution log, Step 3 (issue 1); `xkbcommon-dl` in `Cargo.lock:6886`
- F65. winit picks Wayland when `WAYLAND_DISPLAY` (or `WAYLAND_SOCKET`) is non-empty, else X11 when `DISPLAY` is set.
  — winit 0.30.13 `src/platform_impl/linux/mod.rs:734-766` (cargo registry)
- F66. With no Vulkan ICD installed, `mesa-vulkan-drivers` (lavapipe) made the window open; every screenshot is from
  lavapipe. — observed: verify.md execution log, Step 3
- F67. With Vulkan hidden (`VK_ICD_FILENAMES=/nonexistent.json`), GL on llvmpipe under Xvfb gave a window that stayed
  black, with the app running (`[log] count is 0`) and nothing printed; the runner installs no tracing subscriber.
  — observed: verify.md execution log, Step 4 (issue 5); `shots/step4gl-1.png`
- F68. No display (`env -u DISPLAY`): `svelters-skelter-run: event loop: os error at …/winit-0.30.13/src/platform_impl/linux/mod.rs:765:
  neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.`, exit 1. The prefix is the runner's
  (`svelters-skelter-run.rs:71`); the main.rs in Step 5 uses `hello-skelter:`. — observed: verify.md execution log,
  Step 3 (issue 4)
- F69. Step 3 window: 360×200, titled "Hello skelter", black background, light grey "Hello from Svelte" flush at the
  top-left (the Window default padding 8 is not visible). Closing it: exit 0. — observed: verify.md execution log,
  Step 3; `shots/step3-1-window.png`
- F70. Step 4 window: "Count: 0", "Doubled: 0", a full-width **Increment** button; two clicks → "Count: 2",
  "Doubled: 4". — observed: verify.md execution log, Step 4; `shots/step4-before-window.png`,
  `shots/step4-after-2-clicks-window.png`
- F71. `[log] count is 0` prints while the app loads, before the event loop starts (it printed even in the
  no-display run). Console echo goes to stderr. — observed: verify.md issue 7; F35
- F72. Missing Svelte packages: `…/packages/native-runtime/package.json: package "svelte" of alias target
  "svelte/src/easing/index.js" is not installed (pnpm install)`, exit 1. — observed: verify.md execution log, Step 3
- F73. Compile error with a short path: `Load: Error loading module '/tmp/hs/app/src/Bad.svelte': /tmp/hs/app/src/Bad.svelte:2:4:
  error[unknown_element] (skelter): <push-button> is not an element of the skelter backend` and
  `help: did you mean one of `button`, `radio-button`, `toggle-button`?`; exit 1. — observed: verify.md issue 2
- F74. First build: 388 crates, 2 min 44 s on 4 cores with `CARGO_PROFILE_DEV_DEBUG=0`, 2.4 GB in `target/`.
  — observed: verify.md execution log, Step 3 (issue 11)
- F75. `pnpm install`: 5.3 s, 366 packages, boxed warning "Ignored build scripts: esbuild@0.28.2". — observed:
  verify.md execution log, Step 1
- F76. Restarting after an edit came back at once (compile of the app is fast). — observed: verify.md issue 3
- F77. `cargo run -p hello-skelter` adds the package to `Cargo.lock`; same window, same three log lines, exit 0 on
  close. — observed: verify.md execution log, Step 5
- F78. `--entry fixtures/counter-egui` runs on skelter; the `text-edit` draws no box, text or caret, but typing
  updates `bind:text`. — observed: verify.md execution log, "Where to go next"; `shots/counter-egui-typed2.png`
- F79. The skelter CI workflow builds and tests the skelter crates on `ubuntu-24.04`, `windows-latest` and
  `macos-latest`. — `.github/workflows/skelter.yml:25-30`, `:55`
- F80. `onfoo` on `<button>` → `error[unknown_event] (skelter): <button> has no event `foo` on skelter`. — observed:
  verify.md execution log, Step 4
