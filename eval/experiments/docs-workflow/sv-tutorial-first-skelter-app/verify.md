# Verify: sv-tutorial-first-skelter-app (draft0.md)

Verifier, stage 2. svelters at `6c1ea458` (the commit facts.md names). Read-check against
`/home/user/svelters`; execution in a scratch worktree
`/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/sv-wt` (left in place, with
the tutorial's files and the Step 5 edits to `Cargo.toml`/`Cargo.lock`), target dir
`/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/sv-target`,
`CARGO_PROFILE_DEV_DEBUG=0`, `-j4`. Container: Ubuntu 24.04, 4 cores, Xvfb, Mesa 25.2.8.

**Bottom line: the tutorial works.** Steps 1–5 ran as written. The window opened with the right title and
size, clicking **Increment** updated both labels, and stdout/stderr showed exactly `[log] count is 0/1/2`.
Closing the window ended the process with status 0, and the Step 5 binary behaved the same. The problems are
in the prerequisites and the troubleshooting text, not in the main path.

## Claim-verdict table (facts.md)

| Fact | Verdict | Note |
|---|---|---|
| F1 | ok | `svelters-skelter-run.rs:1-6`, `:16` |
| F2 | ok | `:8-9` |
| F3 | ok | `:58-62` |
| F4 | ok | `:39-46`, `:65-67`; observed: no args → usage, exit 2; load error → exit 1 |
| F5 | ok | `:48-73` |
| F6 | ok | `crates/svelters-skelter/Cargo.toml:21-22`; workspace dep is `default-features = false` (`Cargo.toml:87`), so the feature is needed |
| F7 | ok | |
| F8 | ok | `crates/svelters-modules/src/source/mod.rs:78-108` |
| F9 | ok | `mod.rs:94-103` |
| F10 | ok | `mod.rs:111` |
| F11 | ok | `mod.rs:376` |
| F12 | ok | `mod.rs:63-64`, `:358`; `--dev` run observed working |
| F13 | ok | `mod.rs:131-143`; observed: the app dir needs no `node_modules` |
| F14 | ok | `aliases.rs:22-45`, `:86-88`; message observed verbatim with `node_modules` moved away |
| F15 | ok | |
| F16 | ok | `target.rs:10-21` |
| F17 | ok, but incomplete | Errors are joined (`transform.rs:113-118`), but the message the user sees is **cut at ~256 bytes** by QuickJS (see issue 2) |
| F18 | ok | `transform.rs:120-124` |
| F19 | ok as stated ("a reload"), **misapplied in draft** | The cache is an in-memory `HashMap` per `SourceProvider` (`mod.rs:266`, `:480`); the runner never reloads and does not open the on-disk bytecode cache (no `set_dx`/`set_bytecode_cache` in `crates/svelters-skelter/src`) |
| F20 | ok | |
| F21 | ok | `semantics/backend.rs:15`, `:62` |
| F22 | wrong (count) | `tags` has **34** entries; `kinds` has 37. The draft gives no number, so it's harmless |
| F23 | ok | Window props and doc verified (padding default 8, but see issue 6) |
| F24 | ok | |
| F25 | ok | |
| F26 | ok | |
| F27 | ok | `spec.rs:38-41` area |
| F28 | ok | `codes.rs:73,75,80,84`; `unknown_element` and `unknown_event` observed in real output |
| F29 | ok | `index.js:440`, `:468-476` |
| F30 | ok (content); line ref `:251-266` wrong | The window-open code is `app.rs:178-187` |
| F31 | ok | `backend.rs:763-774` |
| F32 | ok (content); line wrong | `app.rs:209`; observed exit 0 on close |
| F33 | ok (content); line wrong | `app.rs:184`; observed `svelters-skelter: cannot open window #1: Failed to create surface for any enabled backend: {}` |
| F34 | ok | `core.rs:95-114` |
| F35 | ok | `prelude.js:4-22`, `core.rs:626-630`; observed |
| F36 | ok | `core.rs:594` |
| F37 | ok | |
| F38 | ok | `embed/mod.rs:443-454` |
| F39 | ok | |
| F40 | ok | |
| F41 | doubtful | The Cargo line exists, but the built binary links no Wayland/X11/Vulkan library (`ldd` shows only libc, libm, libgcc_s). winit's `wayland-dlopen` unifies with it, so Wayland is dlopened at run time |
| F42 | ok, nuance | Software is ranked last **within** a backend; the first backend with any presentable adapter wins (`gpu.rs:49-59`, `:85-90`) |
| F43 | ok | `FontConfig::testing()` needs no feature |
| F44 | ok | rustup installed 1.97.0 on first `rustc` in the checkout |
| F45 | ok | |
| F46 | ok | |
| F47 | ok | |
| F48 | ok | |
| F49 | ok | |
| F50 | ok | |
| F51 | ok (content); lines `:141-182` off | Test at `host_document.rs:152-200`; it injects `clicked` with `target_event`, not a pointer click |
| F52 | ok | `SKELTER-svelters-skelter.md:185` |
| F53 | ok | |
| F54 | ok | |
| F55 | ok | |
| F56 | not checked (cadatomic) | |
| F57 | ok | |
| F58 | ok (content); line wrong | `app.rs:35-47` |
| F59 | ok | |
| F60 | ok | |
| F61 | wrong source | Not in `packages/build/index.js` (66 lines). It's `packages/native-runtime/vite.js:396` (also `crates/svelters-skelter/src/spec.rs:311`) |
| F62 | ok | |
| F63 | ok | |
| C1–C3 | ok | All compiled and ran |
| C4 | ok | Ran verbatim |
| C5 | ok | Compiled |
| C6 | ok | `cargo check` and `cargo run` passed; it differs from the runner only as stated |
| C7 | ok | `examples/egui-kinds` is the last member |
| C8 | **now verified** | The output was captured from a run (it is not a guess any more) |

## Draft issues

1. **Missing prerequisite on X11: libxkbcommon-x11.**
   Draft (l.27): "On Linux: an X11 or Wayland session, the Wayland client library (skelter's platform crate links it), and a Vulkan or OpenGL driver."
   Verdict: **wrong / incomplete.** On a stock Ubuntu 24.04 container with X11 (Xvfb), the first run panicked
   before any window opened:
   `thread 'main' panicked at …/xkbcommon-dl-0.4.2/src/x11.rs:59:28: Library libxkbcommon-x11.so could not be loaded.`
   Installing `libxkbcommon-x11-0` fixed it. Separately, the binary links no Wayland library (`ldd` shows libc,
   libm, libgcc_s only), because winit loads Wayland, X11 and xkbcommon with dlopen at run time.
   Fix: "On Linux: an X11 or Wayland session with libxkbcommon (and libxkbcommon-x11 on X11; on Debian/Ubuntu
   `libxkbcommon-x11-0`), which are loaded at run time, and a Vulkan driver (on Debian/Ubuntu,
   `mesa-vulkan-drivers` gives you lavapipe)." Drop "links it".

2. **Compile errors are cut off in deep checkouts.**
   Draft (l.114): "A compile error names your file and the problem. The usual one on a first try is `unknown_element` …"
   Verdict: **partly wrong.** The message passes through a QuickJS module-load error and is cut at about 256
   bytes. It also names the path twice: `Load: Error loading module '<path>': <path>:2:4: error[unknown_element] …`.
   In the worktree (long path) the whole output was
   `Load: Error loading module '/tmp/claude-0/…/sv-wt/examples/hello-skelter/app/src/Bad.svelte': /tmp/claude-0/…/sv-wt/examples/hello-skelter/app/src/`
   (262 bytes), with no code and no problem shown. From `/tmp/hs/app` the full text appeared:
   `Load: Error loading module '/tmp/hs/app/src/Bad.svelte': /tmp/hs/app/src/Bad.svelte:2:4: error[unknown_element] (skelter): <push-button> is not an element of the skelter backend` /
   `  help: did you mean one of `button`, `radio-button`, `toggle-button`?`
   Fix: show the real shape of the message, and add one sentence: "The message is cut off after about 256
   characters, so with a deeply nested checkout you may see only the path. Move the app to a shorter path to
   see the full error." (A reader whose checkout is at `~/svelters` will usually see the code but may lose the
   `help:` line.)

3. **The compile cache does not survive a restart.**
   Draft (l.144-145): "the loader caches compiled output by content and only recompiles files that changed."
   Verdict: **wrong in this context.** That cache is an in-memory map inside one `SourceProvider`
   (`crates/svelters-modules/src/source/mod.rs:266`, `:480-498`), and it serves reloads, which the runner never
   does. Every restart recompiles everything, and the runner doesn't open the host's on-disk bytecode cache
   (nothing in `crates/svelters-skelter/src` calls `set_dx` or `set_bytecode_cache`). Fix: drop the clause.
   Compiling the app is fast anyway: the restart came back at once.

4. **"cannot open window" has a different cause, and the process doesn't hang.**
   Draft (l.118-119): "If the window never appears and you see `svelters-skelter: cannot open window`, the platform could not create a window, which usually means there is no display or no usable GPU driver."
   Verdict: **partly wrong.** With no display, the message is a different one:
   `svelters-skelter-run: event loop: os error at …/winit-0.30.13/src/platform_impl/linux/mod.rs:765: neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.` (exit 1).
   `cannot open window` appeared only when no GPU driver was installed:
   `svelters-skelter: cannot open window #1: Failed to create surface for any enabled backend: {}`, and the
   process then exited at once with status **0**.
   Fix: give both messages. "`… event loop: … neither WAYLAND_DISPLAY nor … DISPLAY is set`: no display.
   `svelters-skelter: cannot open window #1: Failed to create surface …`: no usable Vulkan or GL driver.
   Install one (on Ubuntu, `mesa-vulkan-drivers`)."

5. **A software GL driver gave a blank window.**
   Draft (l.28-29): "skelter tries Vulkan first and then GL; a software driver such as lavapipe or llvmpipe works but is chosen last."
   Verdict: **doubtful / partly unsupported.** Lavapipe (Vulkan) worked: every screenshot here comes from it.
   With Vulkan hidden (`VK_ICD_FILENAMES=/nonexistent.json`), skelter fell back to GL on llvmpipe (EGL, Mesa
   25.2.8, Xvfb). The window opened and the app ran (`[log] count is 0`), but the window stayed solid black for
   12 s, and nothing was printed, because the runner installs no tracing subscriber. This may be specific to
   Xvfb (EGL fell back to "surfaceless swrast without DRM"), but on this evidence only lavapipe is known to work.
   "Chosen last" is also only true inside one backend (`gpu.rs:85-90`): a Vulkan lavapipe beats a hardware GL
   driver. Fix: "Vulkan is tried first, then GL. A software Vulkan driver (lavapipe) works. On GL, if the
   window opens but stays black, install a Vulkan driver." Add the black-window case to the troubleshooting
   list.

6. **Step 3 doesn't say what the window looks like.**
   Draft (l.100): "a window titled "Hello skelter" opens with the label in it."
   Verdict: **ok but thin.** Observed: a 360×200 window titled "Hello skelter", dark (black) background, light
   grey text "Hello from Svelte" flush against the top-left corner. The Window spec's default `padding` of 8 is
   not visible (shots `step3-1-window.png`, `step4-before-window.png`). Fix: say "a dark window" so a reader
   doesn't think the black background is a rendering failure. Issue 5 makes this matter, since a failure there
   is also black. Optionally note that the content sits flush against the edges.

7. **The first log line comes at load, before the window opens.**
   Draft (l.147-148): "you should see one line when the window opens and one per click"
   Verdict: **slightly wrong.** `[log] count is 0` prints while the app loads, before the platform loop starts
   (it printed even in the no-display run, before the event-loop error). The lines go to stderr. Fix: "one line
   as the app starts and one per click".

8. **The `text-edit` from counter-egui draws nothing visible.**
   Draft (l.284-287): "`fixtures/counter-egui/src/App.svelte` uses the same element vocabulary with more widgets: a `text-edit` with `bind:text` … Copy pieces of it into your `App.svelte`."
   Verdict: **ok, with a caveat a reader will hit.** `--entry fixtures/counter-egui` runs on skelter (shot
   `counter-egui-window.png`): label, button, slider, checkbox and `<style>` gaps all appear. The `text-edit`,
   though, draws no box, text or caret. Typing into it still updates `bind:text`: "Hello abc" appears in the
   label (`counter-egui-typed2.png`). Fix: add "the text field accepts typing but does not draw itself yet".
   The svelters milestone record lists text services for `text-edit` as pending work.

9. **The "window path is new" bullet is out of date.**
   Draft (l.274-278): "No test in the repository yet opens a real platform window from `svelters-skelter-run --entry`."
   Verdict: **ok as a statement about tests** (grep finds no test or CI that runs the binary). But the bullet
   now undersells the path, which this run exercised end to end on lavapipe under Xvfb. Fix: keep it short
   ("there is no automated test of the windowed runner yet"), or drop it.

10. **Minor lane voice.**
    Draft (l.291-292): "cadatomic hosts its panels this way."
    Verdict: **lane voice / unsupported for this reader.** cadatomic is a sibling project the reader has no
    reason to know, and F56 cites a different repository. Fix: drop the sentence, or name what cadatomic is.

11. **Build time is not given.**
    Draft (l.31): "The first build compiles a large workspace. Give it time."
    Verdict: **ok, could be concrete.** A clean `cargo build -p svelters-skelter --bin svelters-skelter-run -j4`
    (debug info off) compiled 388 crates in **2 min 44 s**, including downloads. It needed 2.4 GB in `target/`
    plus a 137 MB binary. Note also that it builds only the dependencies of `svelters-skelter`, not "a large
    workspace". The repo's `.cargo/config.toml` puts intermediates in `<checkout>/target/build` even when
    `CARGO_TARGET_DIR` is set. Fix: "The first build compiles a few hundred crates: a few minutes on four cores."

12. **pnpm prints a warning.**
    Draft (Step 1): `pnpm install` gives no expected output.
    Observed: it finished in 5 s and printed a boxed warning, "Ignored build scripts: esbuild@0.28.2. Run
    "pnpm approve-builds"…". The warning is harmless for this path, but readers may stop at it. Fix: one
    sentence, "pnpm warns that it ignored esbuild's build script; you can ignore that here."

13. **Wording: what "a directory" means.**
    Draft (l.97-98): "For a directory, the loader looks for the nearest `package.json` and mounts `src/App.svelte`."
    Verdict: **ok, imprecise.** It mounts `<the given dir>/src/App.svelte`, and the nearest `package.json`
    (that dir or any parent) only fixes the app root (`mod.rs:83-101`). A `package.json` with an `embed` table
    changes the entry. Fine for a tutorial.

14. **The `toolkit` field is cosmetic for this path.**
    Draft (l.55-56): "The `svelters` object tells svelters' tools which toolkit the components are written for"
    Verdict: **ok.** The runner overrides it (`svelters-skelter-run.rs:59`), and the draft says so later
    (l.259-260). `fixtures/counter-egui` (toolkit `egui`) ran on skelter unchanged.

Fact-sheet-only problems (they don't reach the draft text): wrong line numbers for F30, F32, F33, F51 and F58;
the wrong source file for F61; the kind count in F22.

## Execution log

Environment notes: no Vulkan ICD, no libEGL, no xdotool and no libxkbcommon-x11 were installed at the start.
I installed `libxkbcommon-x11-0`, `mesa-vulkan-drivers`, `libegl1`, `libegl-mesa0` and `xdotool` with
apt (5 small packages). Screenshots were taken with ImageMagick `import -window root`. Xvfb has no window
manager, so I closed windows with a 10-line C helper that sends `WM_DELETE_WINDOW` (a real close, unlike
`xdotool windowclose`). The driver script is
`/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/drive.sh`.

**Prerequisites.** `rustc --version` in the checkout made rustup install 1.97.0 from `rust-toolchain.toml`
(matches the page). Node v22.22.0 and pnpm 10.33.0 were present. `cc` was present.

**Step 1.** `pnpm install` in the worktree root: done in 5.3 s, 366 packages, plus the esbuild warning (issue
12). Matches the page.

**Step 2.** `mkdir -p examples/hello-skelter/app/src`, then wrote `package.json` and `App.svelte` verbatim.
Nothing to observe. The page needed no extra instruction.

**Step 3.** `cargo run -p svelters-skelter --bin svelters-skelter-run -- --entry examples/hello-skelter/app`
under `xvfb-run -a -s "-screen 0 1280x800x24"`. I built it first with `cargo build -j4` (same target):
2 min 44 s.
- First attempt: panic, `Library libxkbcommon-x11.so could not be loaded` (issue 1). The page has no
  instruction for this.
- After installing libxkbcommon-x11 but with no GPU driver:
  `svelters-skelter: cannot open window #1: Failed to create surface for any enabled backend: {}`, exit 0
  (issue 4).
- After installing mesa-vulkan-drivers (lavapipe): **a window titled "Hello skelter", 360×200, with the
  label.** Matches the page, apart from the dark look (issue 6). Shots: `shots/step3-1.png`,
  `shots/step3-1-window.png` (cropped, 2×).
- Closing with WM_DELETE_WINDOW: the process exited with status 0. **Matches the page.**
- No display at all (`env -u DISPLAY`): event-loop error, exit 1 (issue 4).
- Missing `pnpm install` (simulated by renaming `node_modules`): `…/packages/native-runtime/package.json:
  package "svelte" of alias target "svelte/src/easing/index.js" is not installed (pnpm install)`, exit 1.
  **Matches the page's tip.**
- `<push-button>`: `error[unknown_element]`, but the message was cut off in the long worktree path
  (issue 2). Exit 1.
- With a `.svelte` file as `--entry`, the file mounted. Matches.

**Step 4.** Replaced `App.svelte` with the counter verbatim and ran the same command. The window showed
"Count: 0", "Doubled: 0" and an **Increment** button (`shots/step4-before-window.png`). I clicked the button
centre twice with `xdotool mousemove --window <w> 180 60 click 1`. **Both labels updated** to "Count: 2" and
"Doubled: 4" (`shots/step4-2.png`, `shots/step4-after-2-clicks-window.png`). stderr held exactly:
```
[log] count is 0
[log] count is 1
[log] count is 2
```
**Matches the page.** `--dev` gave the same result. `onfoo` → `error[unknown_event] (skelter): <button> has no
event \`foo\` on skelter`, which matches. (`onclick` is also accepted on `<button>`: skelter has a DOM-like
`click` too.) GL-only check (Vulkan hidden): black window, no error (issue 5, `shots/step4gl-1.png`).

**Step 5.** Created `examples/hello-skelter/Cargo.toml`, added `"examples/hello-skelter"` after
`"examples/egui-kinds"` in the root `members`, and wrote `src/main.rs`, all verbatim.
`cargo check -j4 -p hello-skelter`: OK in 1 min 19 s, with only the existing `svelters-host` dead-code warning.
`Cargo.lock` gained the new package (13 lines), which is expected and not mentioned on the page.
`cargo run -j4 -p hello-skelter` under Xvfb opened the same window. Two clicks gave "Count: 2 / Doubled: 4"
and the same three log lines; close → exit 0. **Matches the page.**
Shots: `shots/step5-1.png`, `shots/step5-2.png`, `shots/step5-after-2-clicks-window.png`.

**Where to go next.** `svelters-skelter-run --entry fixtures/counter-egui` runs on skelter
(`shots/counter-egui-window.png`). `bind:text` works, but the text field draws nothing (issue 8;
`shots/counter-egui-typed2.png`).

## Counts

- Facts F1–F63: ok 52 (several with line-number drift), wrong 2 (F22 count, F61 source file), doubtful 1
  (F41), misapplied in draft 1 (F19), incomplete 1 (F17), not checked 1 (F56). The rest are ok with notes.
  Code blocks C1–C8: 8 ok (C8 now verified by a run).
- Draft issues: 14. Wrong: 2 (#3, #7). Partly wrong: 3 (#1, #2, #4). Doubtful: 1 (#5). Incomplete or needs a
  caveat: 5 (#6, #8, #9, #11, #12). Lane voice: 1 (#10). Fine but imprecise: 2 (#13, #14).
- Execution: Steps 1–5 all ran. The page's main path works once the system has libxkbcommon-x11 and a Vulkan
  driver.
