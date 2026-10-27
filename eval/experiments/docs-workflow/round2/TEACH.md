# Teaching tutorial brief (round 2, 2026-10-27)

Round 1 produced pages that were accurate and useless: click-by-click scripts that told the reader
what to type and never why. The user's verdict: "They explain nothing, give no insight, just
nonsense. I wouldn't even call them tutorials." This round fixes that.

## What a tutorial is here

A tutorial teaches. The reader finishes with a working program **and a mental model** they could
use to predict what the software will do in a situation the page never showed them. Every step
exists to teach an idea; the idea is the point and the step is how the reader meets it.

The model to study first: the Rust book's guessing-game chapter,
`/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/corpus/software-docs/docs-rust-book-src-ch02-00-guessing-game-tutorial-md.md`
(code blocks stripped, so read it for its prose moves). Notice how it:
- introduces one concept at the moment the reader needs it, then uses it immediately;
- explains what each piece of code *does* and *why it's written that way*, line by line where it
  matters;
- shows real output, including errors, and reads the error with the reader;
- lets the reader make a mistake on purpose and learn from what the tool says;
- says what it is not explaining yet and where that comes later.

## Rules

- **Open with the model.** Before the first command: what problem the software solves, what its
  main pieces are, and how they relate, in plain words and at most one diagram (ASCII or a short
  list is fine). The reader should know what they are about to build *into*.
- **Every step says why.** What happens inside when the reader does this, and why the design
  works that way. If a design choice has an alternative the reader would expect (a webview, a
  virtual DOM, re-rendering), say what this does instead and what it buys.
- **Experiments, not just steps.** At least three places where the reader changes something to
  *see* the model: break it on purpose and read the error, add a log line to watch the order of
  events, change one thing and predict the result before running it.
- **Sources.** Code decides what *works*. Design docs (plans/research, decision registers, CLAUDE.md
  decisions, module doc comments) may supply the *why*, but every design claim must match the code
  as built. Where the design describes something not built yet, either leave it out or say plainly
  "not built yet".
  Facts that a reader can check (names, flags, output, behaviour) still go in `facts.md` with
  `path:line`. Explanations of *why* go in with their source: a code comment, a doc comment, a design
  doc section, or "inferred from <code>" when you reasoned it from the code yourself.
- **Honest scope.** Say once, plainly, what isn't there yet (no published crates, Linux tested only,
  …). Don't apologise repeatedly.
- **Voice.** Plain, direct, second person, like a good engineer explaining to a colleague. No lane
  voice (milestone ids, batch names, `§` refs). No template headings ("Overview", "Conclusion",
  "Summary"). No closing paragraph that restates the page. Bold only for UI labels or terms at the
  point they're defined.
- **Length.** Whatever the teaching needs, roughly 3,000–5,000 words of prose. Code blocks extra.

## Output

`facts.md`, `draft0.md`, and `concepts.md` (the 5–10 ideas the reader should leave with, each in
one sentence: the page's learning goals, used to check it).
