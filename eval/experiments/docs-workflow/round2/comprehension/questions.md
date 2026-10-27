# svelters + skelter: comprehension questions

Answer each question in your own words, in a few sentences. Explain the reasoning; you do not need
to quote file names, flags or exact error strings.

The running example is the tutorial's counter app: an `App.svelte` with a `<window>` holding a few
labels and a button, a `count` declared with `$state`, a value computed from it with `$derived`, a
click handler that increments `count`, run first with `svelters-skelter-run` and then hosted from
your own Rust binary.

1. You start the counter with `svelters-skelter-run --entry <your app dir>`. What turns
   `App.svelte` into code that can run, where does that happen, and when? How does that differ
   when you run a bundle built ahead of time instead?

2. You click the `+` button. Describe what happens between the operating system reporting the
   click and your handler's `count++` running: which thread runs the handler, and what does skelter
   do before and after your JavaScript runs?

3. After `count++`, the label reading `Count: {count}` must change. What exactly is updated in
   skelter's scene, which other nodes are touched, and how does the change reach the pixels on
   screen?

4. Your component's top-level element is `<window title="Counter">`. Describe the sequence of steps
   by which that element becomes a real operating-system window showing your labels and button.

5. Where do `$state` and `$derived` actually run: in Rust or in JavaScript, and on whose
   reactivity implementation? What does the Rust side get to see of your component's reactive
   state?

6. svelters runs stock Svelte 5 components, but there is no virtual DOM and nothing diffs a
   rendered tree between updates. Why is no virtual DOM needed, and what does skelter use instead
   to decide what work an update causes?

7. The Svelte compiler you may know produces the same output whatever the page will show. The
   svelters compiler must be told which backend it compiles for (the runner sets it to skelter).
   Why does the compiler need to know the backend, and what does it learn from it?

8. Why does your click handler run synchronously on the UI thread, at the moment the click is
   dispatched, instead of on a separate JavaScript thread or at the next frame? What price does
   that design make you pay?

9. When you host the app from your own binary, the setup passes text- and font-related objects
   to three different places (the backend's document, the widget services, and the compositor).
   Why three, why must they share one font store, and what would you see if you created the backend
   without any text system at all?

10. You add a second `<window>` to `App.svelte` next to the first. Both show a label with
    `{count}`. What do you expect when the app starts, and what happens to the other window, to
    `count`, and to the process when you close one of the two windows? When does the process end?

11. While the counter runs under `svelters-skelter-run --entry … --dev`, you change the label text
    in `App.svelte` and save. What happens in the running app, and why?

12. You add `let size = $derived(count >= 10 ? 'many' : 'few')` and a label showing `{size}`. You
    click from 3 to 4, then stop touching the app. What work does the `size` label cause in
    JavaScript, across the boundary into Rust, and in skelter's tree for that click? And how many
    frames does the app render while you are not touching it?

13. You change one label to an `<input bind:value={name} />`, which is valid Svelte. Now
    `svelters-skelter-run --entry` exits right away with a diagnostic about that tag, and no window
    ever opens. Why is this an error, and at what stage was it caught?

14. In your own binary you create the backend and the embed, load the app, and then call the
    host's `run()` to start the event loop, as you might with another svelters backend. It fails at
    once with "no skelter driver installed". What is wrong with this structure, and what should the
    binary do instead?
