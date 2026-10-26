# Stage 3b — the handprint style loop

You now polish the verified draft for readability and register, with handprint as a critic.
handprint compares the draft with ~380 human documentation pages written before 2022 (Rust book,
Cargo, Node, Rails guides, Vue, Gatsby, Jest, ESLint, CPython, …) and reports dimensions where the
draft falls outside the human band, with a direction (reduce/increase) and spans.

## Run it

    S=/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad
    python3 -I $S/tools/crit.py $S/refs/docs-v0.json <draft>.md --max 12

It strips code blocks first (the corpus was measured without code), prints a summary line, the top
findings and some span snippets, and writes `<draft>.critique.json` with every finding.

## What to act on, and what not to

handprint measures style statistics. It knows nothing about truth or about your reader. So:

- **Facts are frozen.** Never change a fact, label, shortcut, value, identifier, code block or step
  order to satisfy the critic. If a finding can only be fixed by changing a fact, skip it.
- **Ignore findings driven by exact UI labels or code**: e.g. `…` in a real label ("Search
  commands…"), table pipes, slashes in paths, inline-code rates on a GUI page, bold on UI labels
  (bolding exact UI labels is the GUI-docs convention; keep it). Do reduce bold used for emphasis.
- **Never make prose harder to read.** Ignore `read.*` findings that say "increase" (they mean the
  page is simpler than the human average; that is fine).
- **Act on prose habits** the critic is good at: em-dash and parenthesis overuse, asides, bold for
  emphasis, sentence-length monotony, missing hedges or over-boosting (`lex.hyland.*`), weasel or
  wordy words (`lex.doc-style.*`), template phrasing, novel adjective–noun pairs, `It`-openers,
  over-firing negation, list-heavy pages where prose would read better.
- Also check yourself for the five tells a blind judge named on AI docs (handprint misses most):
  template headings (Overview/Introduction/Conclusion/Best Practices), list-closers that restate the
  list ("Each of these steps plays a crucial role…"), filler intensifiers ("crucial", "seamless",
  "simply"), "It's <adjective> to …" openers, and a closing paragraph that restates the page. And:
  a sentence that explains what the previous sentence already showed.

## The loop

- At most **3 passes** (draft2.md, draft3.md, draft4.md). Each pass: run the critic, choose the
  findings worth acting on, revise, re-run. Record in `log.md` under "Pass N — handprint": the
  summary line before/after, which findings you acted on, which you skipped and why (one line each).
- **Churn guard:** if a pass's new findings barely overlap the previous pass's, you are chasing the
  report; stop. **Keep the best version, not the last.** Best = fewest High/Medium findings in the
  families you can legitimately act on, with no fact changed and no loss of clarity.
- Copy the chosen version to `final.md` and say in log.md which pass it is and why.
- Finally, diff final.md against draft1.md and confirm in log.md that no fact, label, value or code
  block changed (list any sentence where meaning shifted, and fix it back).

Don't use `p_same_author` as a target; it is informational and known to rank AI drafts above human
pages at this length.
