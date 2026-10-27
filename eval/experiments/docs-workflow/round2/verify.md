# Verify: "A Svelte counter in a native window, from the inside" (draft0.md, round 2)

Verifier, run by execution. svelters `6c1ea458` (the commit facts.md names), in a fresh worktree
`/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/sv-wt2` (left in place: it holds
the tutorial's files in their final state, plus the `Cargo.toml`/`Cargo.lock` edits). Target dir
`/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/sv-target`,
`CARGO_PROFILE_DEV_DEBUG=0`, `-j4`. Container: Ubuntu 24.04, 4 cores, Xvfb (`-screen 0 1280x800x24`), Mesa
25.2.8 lavapipe, rustc 1.97.0, Node 22.22.0, pnpm 10.33.0. Real input only: `xdotool mousemove` + `click`,
windows closed with `WM_DELETE_WINDOW`, screenshots with `import -window <id>`.

Run artefacts: driver `runs/drive.sh`; per-run stdout/stderr `runs/<name>.std{out,err}`; screenshots
`shots/<name>-*.png` (all under `/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/exp2/sv-tutorial/`).
The exact Rust files as compiled at each stage: `runs/main-K8.rs` (own binary), `runs/main-K9.rs` (dump +
traffic), `runs/main-K11.rs` (light theme), `runs/main-K12.rs` (logger). Compiled JS: `runs/App.js`.

**Bottom line.** Every code block compiled and ran as written: the runner, the counter, the logging variant,
the crate, the tree dump, the traffic counters, the `<style>` panel, `flex-grow`, the light-theme swap and
the logger. The tree dump and all three traffic predictions came out exactly as the page says, and so did
the event order. What's wrong is in the commentary around them. The compiler-output experiment describes code
the compiler doesn't emit for skelter (no `$$P_…` ids, no `$n.ids`, no `$n.setp`). The sample error message
has the wrong column and could not be printed with the path it shows. The hover prediction says the button
"visibly changes", and in this build it doesn't. "The platform speaks first" has the order backwards. Three
explanations are right but too simple: the "button is not touched" frame claim, the "only through events" model
claim and the `name` derived claim.

## Execution log

### Setting up
- `git -C /home/user/svelters worktree add …/sv-wt2 HEAD` → detached `6c1ea458`.
- `pnpm install` → 1.9 s, `Packages: +366`, then the boxed warning `Ignored build scripts: esbuild@0.28.2. Run
  "pnpm approve-builds"…`. **Match** (l.69-70).
- Build `svelters-skelter-run` + `svelters-compile` from cold (the old warm cache was in the deleted
  worktree's `target/build`, because `.cargo/config.toml` sets `build-dir = "{workspace-root}/target/build"`):
  **2 min 07 s** on 4 cores. **Match** for "about three minutes" (l.113-114).

### A window from a component (l.77-158)
- Wrote `package.json` and `App.svelte` verbatim. Ran
  `cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry examples/hello-skelter/app`.
- The window appeared 0.8 s after launch, `Geometry: 360x200`, title `Hello skelter`. It was black, with
  "Hello from Svelte" in light grey flush top-left (`shots/s1-hello-0.png`; histogram: 71,456 px `#000000`,
  text anti-aliasing up to `#A3A3A3`). Closing it → exit 0. **Match** (l.114-116).

### Experiment: break it on purpose (l.160-201)
- `<push-button>` from the worktree (long path), runner: exit 1, and the whole output was
  `Load: Error loading module '/tmp/claude-0/…/sv-wt2/examples/hello-skelter/app/src/App.svelte': /tmp/claude-0/…/sv-wt2/examples/hello-skelter/app/sr`
  (262 bytes; no code, no problem). This is the truncation the page warns about. **Match** for the warning.
- The same file under `/tmp/hs2/app` (short path), runner:
  ```
  Load: Error loading module '/tmp/hs2/app/src/App.svelte': /tmp/hs2/app/src/App.svelte:3:6: error[unknown_element] (skelter): <push-button> is not an element of the skelter backend
    help: did you mean one of `button`, `radio-button`, `toggle-button`?
  ```
  exit 1. **Mismatch: column `3:6`, not `3:4`** (issue 2).
- `svelters-compile --backend skelter examples/hello-skelter/app/src/App.svelte` →
  `examples/hello-skelter/app/src/App.svelte:3:6: error[unknown_element] (skelter): …` + the help line, exit 1.
  The path is printed as typed. **Match** (l.194-199), column aside.
- `onfoo={() => {}}` on the label: CLI `…App.svelte:3:12: error[unknown_event] (skelter): <label> has no event \`foo\` on skelter`,
  exit 1; runner the same behind `Load: Error loading module '…':`. **Match** (l.189-190).

### Making it react (l.203-269)
- Counter verbatim. The window showed "Count: 0", "Doubled: 0" and a full-width **Increment** button
  (`shots/s2-pre-0.png`). `[log] count is 0` was already on stderr when the window appeared (1 stderr line at
  window time). **Match** (l.237-239).
- Two clicks at the button centre (180,60): labels "Count: 1 / Doubled: 2", then "Count: 2 / Doubled: 4"
  (`shots/s2-counter-strip.png`). stderr was exactly `[log] count is 0`, `[log] count is 1`, `[log] count is 2`.
  stdout empty. **Match** (l.227-235).

### Experiment: read what the compiler made (l.271-303)
- `cargo run -p svelters-compiler --bin svelters-compile -- --backend skelter …/App.svelte > …/App.js`: exit 0,
  after a **24 s** build of the CLI (even though the runner was already built). The output is 28 lines
  (`runs/App.js`):
  ```js
  const $$n1 = $n.toplevel("window", "present");
  $n.set($$n1, "title", "Hello skelter");
  …
  const $$t0 = $n.instantiate($$T0, $n.slot($$n1, null), [() => 8]);
  const $$n3 = $n.at($$t0, 1);
  …
  $.template_effect(() => {
      $n.set($$n3, "text", `Count: ${$.get(count) ?? ''}`);
      $n.set($$n4, "text", `Doubled: ${$.get(doubled) ?? ''}`);
  });
  $n.template_listen($$t0, [() => $.update(count)]);
  …
  const $$T0 = $n.template("299f0516aa46760e", [["box", null, [["setv", "gap", 0]], [1, 2, 3], null], ["label", …], ["label", …], ["button", null, [["set", "text", "Increment"]], [], null]], [[3, "clicked", 0]]);
  ```
  **Match:** `$n.toplevel("window", …)`, the `$n.template` table and `$n.instantiate`, `$.template_effect`,
  `$n.template_listen` with `$.update(count)`, `$.user_effect`.
  **Mismatch:** there is no `const $$P_Label_text = 135398928;`, no `$n.ids(…)`, and no `$n.setp(…)`. Props are
  written by **name**: `$n.set($$n3, "text", …)` (issue 1).

### Experiment: watch the order of events (l.305-377)
- Logging variant verbatim, one click. stderr:
  ```
  [log] template reads count 0
  [log] template reads doubled 0
  [log] effect: count is 0
  [log] handler: count was 0
  [log] handler: count is now 1
  [log] template reads count 1
  [log] template reads doubled 2
  [log] effect: count is 1
  ```
  The last five lines are **exactly** the page's block (l.346-352). The first three print at startup, and the
  page doesn't mention them (issue 10).
- Added `let name = $state('Ada')` and `<label>Name: {show('name', name)}</label>`. Two clicks: no
  `template reads name` after startup. **Match** for the prediction. The compiled code shows the reason is a
  different one: `() => show('name', name)`. There is no `$.get(name)`, because `name` is never reassigned (issue 9).
- Plain `{name}`: compiles to `$n.set($$n5, "text", \`Name: ${name ?? ''}\`)` inside the template effect.
  It re-runs, and `set` drops it (`packages/native-runtime/src/index.js:696-697`). **Match** (l.373-377). It is
  confirmed by traffic too (below).

### Hosting it from your own binary (l.379-472)
- `Cargo.toml`, the `members` edit and `main.rs` verbatim (`runs/main-K8.rs`). `cargo build -p hello-skelter`
  OK, with only the existing `svelters-host` dead-code warning. `Cargo.lock` gained the package (the page doesn't say).
  The run gave the same window, and one click gave the same log lines. Exit 0 on close. **Match** (l.450-451).

### Experiment: look at the tree, and count the crossings (l.474-565)
- `dump` and the three edits verbatim (`runs/main-K9.rs`): compiled with no new warnings. Before the window, stderr:
  ```
  [log] count is 0
  <window> handle 1
    <box> handle 2
      <label> handle 3 text=Some(Str("Count: 0")) change=Some(Layout)
      <label> handle 4 text=Some(Str("Doubled: 0")) change=Some(Layout)
      <button> handle 5 text=Some(Str("Increment")) change=Some(Layout)
  ```
  **Exact match** (l.531-537). The page doesn't say that the `[log]` line comes first.
- Traffic, counter app:
  | run | output |
  |---|---|
  | 1. close untouched | `Traffic { syncs: 0, posts: 0, changes: 0, checkpoints: 8, frames: 0, creates: 5, tree: 4, sets: 7, gets: 3, calls: 1, listens: 1, styles: 7 }` |
  | 2. two clicks | `… posts: 2, … checkpoints: 27, … sets: 11, …` (rest identical) |
  | 3. 5 moves over/off, no click | `… posts: 0, … checkpoints: 117, … sets: 7, …` (rest identical) |

  **Match** on every prediction: posts +2, sets +4, and hover adds nothing to posts or sets. The startup reading
  holds too: creates 5, tree 4, listens 1, calls 1 (`present`), gets 3. `checkpoints` grows with movement.
  With an unchanged `{name}` label (`s9-name0`/`s9-name2`): sets 8 → 12 for two clicks, so the label adds
  nothing. **Match** (l.564-565).
- **Mismatch: the button did not visibly change on hover.** The fill sampled at (20,60) was `gray(49)`
  (`button-bg` = `#313131`) over and off the button in every move-only run (`s2-hover-*`, `s2-hover2-*`,
  `s5-t3-*`). The same held with timer-driven frames running (`probe-hover-timer-*`) and with an `onpointermove`
  listener that did receive the moves (`probe-move`). The hover fill `gray(59)` (≈ `button2-hover` `#3c3c3c`)
  appeared only after a click. It then **stayed** after the pointer left (`s2-counter-away.png`) (issue 3).

### Why the window is black (l.567-612)
- `.panel { background-color: #2b2b2b; padding: 12px; }` and `class="panel"`: a grey panel as wide as the window
  and only as tall as its content, with the content inset 12 px and black below (`shots/s6-panel-0.png`).
  **Match.** `styles` went from 7 to 8.
- `flex-grow: 1;`: the panel fills the window (`shots/s6-grow-0.png`). **Match.**
- Light theme (`skelter-widgets` dependency plus the two lines, `runs/main-K11.rs`): compiled. Dark text, light
  button, black background (`shots/s7-light-0.png`; the labels are hard to read). **Match.**
- Extra check, not in the page: setting `CompositorConfig.clear` to `Color::from_rgba8(0, 0, 128, 255)` turned the
  whole background navy (`shots/s10-clear-0.png`, 61,316 px `#000080`). The black **is** the compositor's clear
  colour. The explanation is right, and this makes a better experiment than the page has (issue 15).
- Code check: `window` is a `Placeholder` (`crates/svelters-skelter/src/kinds.rs:4-7`, `:538-549`, `record` is
  empty). `CompositorConfig::new` sets `clear: Some(Color::TRANSPARENT)` (`crates/skelter-compositor/src/lib.rs:84-85`,
  `:103`). Platform windows are opaque (`transparent: false`, `crates/skelter-platform/src/api.rs:217`). The
  placeholder lays out from the computed style only (`Kind::layout_style` default → `cx.default_layout()`,
  `crates/skelter-core/src/kind.rs:497-500`), so its `padding` prop (default 8 in `skelter.json`) is stored and
  never read. **The explanation holds.**

### When something goes wrong (l.645-696)
- No display (`env -u DISPLAY`): stderr shows `[log] count is 0` **first**, then
  `svelters-skelter-run: event loop: os error at …/winit-0.30.13/src/platform_impl/linux/mod.rs:765: neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.`,
  exit 1 (issue 7).
- `ldd hello-skelter`: only `linux-vdso`, `libgcc_s`, `libm`, `libc`, `ld-linux`. **Match** ("links none of the
  display libraries").
- `node_modules` moved away: `…/packages/native-runtime/package.json: package "svelte" of alias target "svelte/src/easing/index.js" is not installed (pnpm install)`,
  exit 1. **Match.**
- Logger verbatim (`runs/main-K12.rs`, `log = "0.4"`): compiled. One run with a click:
  ```
  [log] count is 0
  <window> handle 1 … (dump)
  [WARN] winit::platform_impl::linux::x11::xdisplay: error setting XSETTINGS; Xft options won't reload automatically
  [WARN] winit::platform_impl::linux::x11::util::randr: XRandR reported that the display's 0mm in size, which is certifiably insane
  [ERROR] wgpu_hal::gles::egl: EGL 'eglInitialize' code 0x3001: DRI2: failed to load driver
  [ERROR] wgpu_hal::gles::egl: EGL 'eglInitialize' code 0x3001: DRI2: failed to load driver
  [INFO] skelter_compositor::gpu: skelter-compositor: llvmpipe (LLVM 20.1.2, 256 bits) on Vulkan (Cpu)
  [log] count is 1
  Traffic { … posts: 1, … sets: 9, … }
  ```
  **Match** for the INFO line. **Not mentioned:** two WARN and two ERROR lines on a run that works (issue 5).
- Vulkan hidden (`VK_ICD_FILENAMES=/nonexistent.json`), with the logger: the window opened and stayed black with
  no text. The log showed the Vulkan loader errors, the EGL errors, and then
  `[ERROR] skelter_compositor::gpu: skelter-compositor: no GPU adapter for backends [Backends(VULKAN), Backends(GL)]`.
  Close → exit 0. **Match** for "shows up as an `[ERROR]` line". The same case through the runner (no logger)
  printed only `[log] count is 0`: a silent black window. So a silent black window also means **no adapter at
  all**, not only software GL (issue 6).

## Explanations checked against the code

| Claim (draft line) | Verdict | Evidence |
|---|---|---|
| Main thread runs winit, the tree and QuickJS; host starts no threads (18-21) | ok | `crates/skelter-platform/src/backend.rs:44-58`; `crates/svelters-host/src/lib.rs:18-19`; `crates/skelter/src/app.rs:1-2` |
| Compositor is its own thread owning wgpu, draws what was committed (30-31, 43) | ok | `crates/skelter-compositor/src/lib.rs:3-7`; `crates/skelter/src/backends.rs:377-391` |
| Host ops are direct calls, nothing queued (123-125) | ok, nuance | `core.rs:1077` "Ops reach the backend directly while JS runs". The call goes through the `svl_backend` C-ABI vtable (`svelters-host/src/lib.rs:1-2`), and values are converted, not serialised |
| JS reaches the tree only via host ops; tree reaches JS "only through events" (46-48) | **oversimplified** | Also `host.prop_changed` for observed native-owned props (`crates/svelters-skelter/src/deliver.rs:141-151`; the `changes` counter). That is how `bind:` gets values back (issue 8) |
| Startup order 1-6 (120-143) | ok | runner `svelters-skelter-run.rs:48-73`; `app.rs:1-15`, `:173-188`, `:99-107`; `backend.rs:763-774` |
| Retained, re-record only dirty nodes; commit carries only re-recorded lists (147-149) | ok | `skelter-core/src/frame.rs:1-15`; `skelter-base/src/commit.rs:16-29` |
| egui per-frame cost is the main reason skelter exists (149-151) | ok (design) | `plans/research/skelter/README.md:70-73` |
| Spec generated from Rust kinds; catches tags at compile (185-188) | ok | `semantics/backend.rs:14-15`; observed |
| 256-char cut "through QuickJS's module-loading error" (192-194) | ok, imprecise | QuickJS formats every thrown error into `char buf[256]` (`rquickjs-sys-0.14.0/quickjs/quickjs.c:8431-8434`). rquickjs throws the loader's `Error::Loading` through it (`rquickjs-core-0.14.0/src/result.rs:317-325`). 255 **bytes**. `Load:` is svelters-host's `ErrorKind::Load` (issue 11) |
| Click path 1-6 (245-259) | ok | `skelter/src/app.rs:817-847` (discrete input dispatched at once); `skelter-core/src/input/dispatch.rs:1-4`, `:233-270`; `button.rs:247-280`; `deliver.rs:69-78`, `:150-200`; `core.rs:1063-1079`; `native-runtime/src/index.js:1229-1305`; observed posts +1/click |
| Only changed values cross; `set_prop` returns at once on an equal value (264-265, 375-377) | ok | `index.js:672-708` (`set`, `last` at `:696-697`); skelter has no `SVL_CAP_DEDUPE` (`svelters-abi/src/safe.rs:1692`), so JS dedupes; `document.rs:837-840`; observed sets +2/click |
| Layout class → re-measure, relayout only if size changed, repaint (266-268) | ok | `skelter-base/src/change.rs:11-12`; `document.rs:969-980` |
| "The button and the box are not touched" (269) | **misleading** | The click changes the button's press/hover state, so it restyles and re-records. Observed: the fill went 49 → 59 after a click (issue 4) |
| Compiler emits `$$P_Label_text`, `$n.ids`, `$n.setp` (286-291) | **wrong** | Not in the output. `skelter.json` nests `spec_hash` under `"ids"` (`crates/svelters-skelter/src/spec.rs:400`; asserted by `tests/spec.rs:47`), but the compiler reads a top-level `spec_hash` (`svelters-compiler/src/semantics/spec.rs:62`, `backend.rs:272-274`), so `SpecIds::new` returns `None` (`compile.rs:261-264`) and the ids path never turns on for skelter. A svelters bug; `egui.json:18493` has it top-level (issue 1) |
| Templates expanded by the runtime; skelter doesn't build natively (299-303, 708-709) | ok | `native-runtime/src/template.js:16-22`, `:38-45`; no `supports_templates` override (`svelters-abi/src/safe.rs:1674` default false; grep finds none in `crates/`) |
| Handler first, then the template effect, then `$effect`, in the checkpoint's drain (354-357) | ok | observed exactly; `core.rs:1063-1079` |
| Call expressions get their own derived (371-373) | ok in general, **not shown by this example** | `codegen/component.rs:2063-2068`; but `name` compiles to a plain value (issue 9) |
| Handles count up and are never reused (539-541) | ok | `backend.rs:73-91`; observed 1-5 |
| `HostConfig::default()` 10 s turn budget (461-463) | ok | `core.rs:103`; `EmbedConfig::app` takes it (`embed/mod.rs:100-102`) and `with_backend` applies it (`:392`) |
| `Span` is the refresh interval until the platform reports one (468-469) | ok | `skelter-compositor/src/lib.rs:73-74` |
| Checkpoint after every platform callback (563-564) | ok | `app.rs:175-177`; `embed/mod.rs:633-643`; observed 8 / 27 / 117 |
| Hover handled in Rust; "the button visibly changes" (560-561) | **half wrong** | In Rust: yes (`dispatch.rs:342-360`, `button.rs:247-258`, nothing posted). Visible: not in this build (issue 3) |
| Black window: placeholder, transparent clear, theme only colours (571-579) | ok | see the section above; confirmed by the clear-colour experiment |
| Box paints its CSS box; column flexbox; window root at window size (581-599) | ok | `box_.rs:1-8`; `cascade.rs:513-516`; observed |
| Native-owned props refuse stale JS writes (624-627) | ok | `document.rs:820-832`; `backend.rs:663-670` |
| Kinds / behaviours / states; cadatomic uses all three (629-643) | ok | `kinds.rs:575-590`; `behavior.rs:1-16`; `backend.rs:563-569`; cadatomic `b350c24`: `cadatomic-ui/src/skelter/embed.rs:29`, `:71`, `mod.rs:57-60`, `cadatomic-app/src/skelter/tree.rs:67` |
| "The platform speaks first" (649) | **wrong order** | The app loads before `skelter_platform::run`. Observed: `[log]` before the display error (issue 7) |
| Vulkan before GL on Linux (694) | ok | `skelter-compositor/src/gpu.rs:49-59` |
| skelter CI on Linux, Windows, macOS (702-703) | ok | `.github/workflows/skelter.yml:29` |

## Issues

1. **The compiler-output experiment describes code that isn't emitted.**
   Draft (l.286-291): "Constants such as `const $$P_Label_text = 135398928;` and one `$n.ids("skelter", …)` call.
   Props are named by numbers, not strings. … the `$n.ids` call checks once per module that the two agree." and
   "A `$.template_effect(…)` whose body writes the two labels' text with `$n.setp(…, $$P_Label_text, …)`."
   Verdict: **wrong.** The real output has no `$$P_`/`$$K_` constants, no `$n.ids`, no `$n.setp`. The effect body is
   `$n.set($$n3, "text", \`Count: ${$.get(count) ?? ''}\`)`. The cause is a svelters bug: skelter's spec puts
   `spec_hash` inside `"ids"` (`crates/svelters-skelter/src/spec.rs:400`), and the compiler only turns ids on when
   the spec has a top-level `spec_hash` (`crates/svelters-compiler/src/compile.rs:262`,
   `semantics/backend.rs:272-274`, `semantics/spec.rs:62`).
   Fix: replace the two bullets with what the reader will see:
   > - Calls such as `$n.set($$n3, "text", …)`. Props travel by name. The spec also gives every prop a
   >   numeric id: search `skelter.json` for `135398928` and you'll find `Label` → `props` → `text`, next to
   >   `"change": "layout"`. The compiler has a mode that names props by those ids instead of strings, but it
   >   isn't switched on for skelter yet.
   > - A `$.template_effect(…)` whose body writes the two labels' text with `$n.set(…, "text", …)`. That is the
   >   one effect that re-runs on a click.

   Also report the bug to the coordinator, with the three paths above and the line from `tests/spec.rs:47`
   (which pins the nested form).

2. **The sample error message is wrong twice.**
   Draft (l.176-177): `Load: Error loading module '/home/you/svelters/…/App.svelte': /home/you/svelters/…/App.svelte:3:4: error[unknown_element] (skelter): … help: did you mean one of …`
   Verdict: **wrong.** (a) The column is `3:6` (the tag name's column; observed from both the loader and the
   CLI). (b) With that 56-character path the message is 310 bytes, and QuickJS keeps 255, so the reader would see
   `…the skelter backend` followed by `  help: did you` and nothing more. The block shows a message that can't be printed.
   Fix: show the real cut message and let it lead into the warning paragraph:
   ```text
   Load: Error loading module '/home/you/svelters/examples/hello-skelter/app/src/App.svelte': /home/you/svelters/examples/hello-skelter/app/src/App.svelte:3:6: error[unknown_element] (skelter): <push-button> is not an element of the skelter backend
     help: did you
   ```
   Then: "The help line is cut off. Why, and how to see the rest, comes in a moment." (Or keep the full
   message but say it was taken from a short path such as `/tmp/hs/app`.)

3. **Hovering doesn't visibly change the button in this build.**
   Draft (l.560-561): "hovering should add nothing to `posts` or `sets`, even though the button visibly changes as
   the pointer enters it."
   Verdict: **wrong as observed.** Moving the pointer onto and off the button never changed its fill in five
   runs. That included runs where timer-driven frames were drawing and where an `onpointermove` listener proved
   that the moves reached skelter-core. The hover colour appears only after a click, and it stays after the
   pointer leaves until the next click (`s2-counter-away.png`). The counters part is right.
   Fix: "hovering should add nothing to `posts` or `sets`. skelter-core and the button's kind track hover in Rust
   (that's what the `checkpoints` growth is: platform callbacks), and no listener asked for pointer events."
   Drop "visibly". Report the hover repaint to the coordinator as a probable skelter bug: hover state from
   coalesced pointer moves isn't repainted until a discrete input arrives.

4. **"The button and the box are not touched" is misleading on a click.**
   Draft (l.269). Verdict: **misleading.** `count++` doesn't touch them, but the click itself changes the button's
   press and hover state bits, so the button restyles and re-records in the same frames (its fill goes from 49
   to 59 after a click). Fix: "`count++` doesn't touch the button or the box. The button does repaint, but for
   its own reason: its press and hover look, which Rust handled before any JavaScript ran."

5. **The logger prints warnings and errors on a working run.**
   Draft (l.691-693) describes one INFO line. Verdict: **incomplete.** Under Xvfb the same run also printed two
   `[WARN] winit::…` lines and two `[ERROR] wgpu_hal::gles::egl: EGL 'eglInitialize' code 0x3001: DRI2: failed to load driver`
   lines, before the INFO line that says Vulkan was chosen. A reader told that failures "show up as an `[ERROR]`
   line" will think something broke. Fix: show the real block, and add "wgpu probes GL as well; its EGL errors are
   harmless when the INFO line names Vulkan". Or narrow the filter to `m.target().starts_with("skelter")`: that
   still shows the adapter line and `skelter-compositor: no GPU adapter …` (both target `skelter_compositor::gpu`)
   and drops the noise.

6. **A silent black window has a second cause.**
   Draft (l.655-657): "A window that opens but stays black, with no text at all, was seen with Mesa's software GL
   under Xvfb". Verdict: **incomplete.** With no usable adapter at all (Vulkan hidden, EGL failing), the window
   also opened and stayed black and silent. The runner printed only `[log] count is 0`. With the logger, the
   cause is one line: `[ERROR] skelter_compositor::gpu: skelter-compositor: no GPU adapter for backends [Backends(VULKAN), Backends(GL)]`.
   Fix: "A window that opens but stays black with no text means the compositor has no working GPU path, either
   none at all or a GL driver that doesn't draw. The runner says nothing; install the logger below to see which."

7. **"The platform speaks first" has the order backwards.**
   Draft (l.649). Verdict: **wrong.** `main` loads and mounts the app before `skelter_platform::run`. So loader and
   compiler errors, and your own `[log]` lines, come out before any platform error. Observed with no display:
   `[log] count is 0`, then the `neither WAYLAND_DISPLAY …` line. Fix: order the list as loader/compiler, your
   JavaScript, platform, compositor, and say "in that order, because the app loads before the event loop starts".

8. **"The tree reaches JavaScript only through events" leaves out change notices.**
   Draft (l.47-48). Verdict: **oversimplified.** Native-owned props that JS observes (a slider's value, a text
   field's text under `bind:`) come back as `prop_changed` calls (`crates/svelters-skelter/src/deliver.rs:141-151`).
   That is the `changes` counter the page prints and never explains. Fix: "…only through **events** (the ones
   JavaScript listens to) and **change notices** for the props it binds." Mention `changes` once in the traffic
   section.

9. **The `name` experiment doesn't show what the text says it shows.**
   Draft (l.371-373): "Each function call in a template is compiled into a derived value of its own, which
   re-computes only when what it read changes. `name` didn't change, so `show('name', …)` isn't called."
   Verdict: **true in general, but this example doesn't exercise it.** `name` is never reassigned, so the compiler
   makes it a plain variable: `() => show('name', name)`, with no `$.get`. Nothing could ever change it. Fix
   (verified): add `<button onclicked={() => name = 'Grace'}>Rename</button>` after Increment. Then the compiled
   thunk is `() => show('name', $.get(name))`. **Increment** logs no `name` line, and **Rename** logs only
   `[log] template reads name Grace`, with no count, doubled or `$effect` line (`runs/s11-name.stderr`). That
   shows the derived memoisation and the effect's dependency tracking from both sides.

10. **The event-order output leaves out the startup lines.**
    Draft (l.346-352). Verdict: **incomplete.** Three lines print before the window appears:
    `template reads count 0`, `template reads doubled 0`, `effect: count is 0`. They're worth showing: mount runs
    the template effect before your `$effect`, the same order as a click.

11. **Who says `Load: Error loading module`.**
    Draft (l.180-181, 192): "`Load: Error loading module '…'` is QuickJS" and "QuickJS's module-loading error, which
    shortens it to about 256 characters". Verdict: **imprecise.** `Load:` is svelters-host's error kind
    (`ErrorKind::Load`). "Error loading module '…'" is rquickjs's wording (`rquickjs-core-0.14.0/src/result.rs:453-461`).
    The cut happens because QuickJS formats every error message it throws into a 256-byte buffer (`quickjs.c:8431`).
    It's bytes, so non-ASCII paths lose more. Fix: "…the JavaScript engine formats every error message into a
    256-byte buffer, so the message is cut after about 255 bytes."

12. **The compiler CLI needs its own build.**
    Draft (l.198, 276). Verdict: **minor gap.** The first `cargo run -p svelters-compiler --bin svelters-compile`
    compiled for about 24 s, even after the runner was built. Say "the first run builds the CLI (under a minute)".

13. **The light-theme snippet isn't "the same installation", and "no panel rule" is vague.**
    Draft (l.601-608). Verdict: **minor.** `install_widget_services` picks the platform with `cfg!(target_os)`
    (`app.rs:38-45`), and the snippet hard-codes `Platform::Linux`. That's fine for this Linux-only page, but say so.
    "With no panel rule": tell the reader to delete the `<style>` block. The snippet ran and the result matched
    (`s7-light-0.png`). Also tell them to switch back afterwards, or the later logger run is still light.

14. **Counters and change classes that are printed but never explained.**
    Verdict: **teaching gap.** `styles` (7 at startup, 8 with a `<style>` block), `syncs`, `changes` and `frames`
    appear in the Traffic line, and only some get a word. The page introduces change classes, but only `Layout`
    ever appears. `Paint`, `Property` and `Structure` are never named, though concepts.md goal 5 lists them.
    Fix: one sentence naming the four classes where `change=Layout` first shows up (l.542). One clause each for
    `syncs` ("listeners that must answer before the native default, such as a cancelable `click`") and `changes`
    (issue 8).

15. **The black-window explanation has no experiment that isolates it.**
    Verdict: **teaching gap (claim verified).** The section asserts that the clear colour is what shows, and the
    reader can't check that. The page's own binary can. This was run, and the window turned navy:
    ```rust
    let spawn = skelter::compositor(|clock| {
        let mut c = skelter_compositor::CompositorConfig::new(clock, skelter_base::time::Span::from_micros(16_667));
        c.clear = Some(skelter_base::color::Color::from_rgba8(0, 0, 128, 255));
        c
    }, glyphs);
    ```
    That's a one-line prediction the reader can make and check, which is what the brief asks for.

## Teacher's read

- **Where a reader can't check a claim.** That only the two labels re-record (l.268). The commit carrying two
  display lists. Damage-limited redraw. "One transaction". Nothing on the page makes these visible. The traffic
  counters show the JS↔Rust side well, but nothing shows the frame side. Either say once that these are internals
  the reader takes on trust, or point at the latency/trace export (`crates/skelter/src/latency.rs`) as the place to
  look. Don't add more assertions.
- **Where a newcomer gets lost.** The compiled-JS bullets name `$n.slot`, `$n.at`, `$n.append` and a thunk array
  `[() => 8]` without a word. Once issue 1 is fixed, one sentence covers it: "the table holds the static parts; the
  array holds the values the compiler couldn't fold (`gap={8}` is an expression)". The partial snippets in the
  `name` step (`// ...as before, plus:`) are fine for this audience.
- **Introduced but not used:** `syncs`/`changes`/`frames`/`styles` (issue 14), and change classes other than
  Layout. **Used well:** the break-it error, the dump, and the three traffic runs. Those are the page's best
  teaching, and they all ran exactly as written.
- **Length:** 4,661 words of prose outside code blocks. That's inside the brief's 3,000–5,000.

## Counts

- Code blocks: 12 run (K1-K4, K4b with its `name` addition, K5 CLI, K6-K8 crate, K9 dump and traffic, K10 panel
  and `flex-grow`, K11 light theme, K12 logger). **12/12 compiled and did what was asked.**
- Predicted outputs: 14 checked. **Exact or substantive match: 10:** the first window, `[log]` order, both
  break-it diagnostics (apart from the column), the event order, the `name` silence, the tree dump, the traffic
  deltas for clicks and hover, the panel, `flex-grow`, the light theme, the adapter INFO line.
  **Mismatch: 4:** the compiler-output bullets, the error column and cut, the hover look, extra logger lines.
- Explanations checked: 27. ok 19, ok with nuance 3, oversimplified or misleading 3 (#4, #8, #9), wrong 2
  (#1 ids, #7 order). Plus one half-wrong prediction (#3).
- Issues: 15. Wrong: 4 (#1, #2, #3, #7). Misleading or oversimplified: 3 (#4, #8, #9). Incomplete: 4 (#5, #6,
  #10, #12). Imprecise: 2 (#11, #13). Teaching gaps: 2 (#14, #15).
- svelters bugs found on the way, for the coordinator: (a) skelter's spec nests `spec_hash`, so the compiler never
  turns on stable ids for skelter (#1). (b) The button's hover look doesn't repaint on pointer moves alone (#3).
- Not run: Wayland, a real GPU, the `libxkbcommon-x11` panic (round 1 observed it; the package is installed
  here), and the "cannot open window #1" message (round 1 observed it; this container's GL path fails differently).
