# Docs experiment brief (2026-10-26)

We are testing the documentation workflow designed in /home/user/handprint/DOCS.md and
/home/user/handprint/DOCS2.md. Four documents are written by agents, checked against the code,
then polished with handprint (a stylometry critic) and judged blind. The user will read the
finished documents. They should be genuinely useful to a person who has never used the software.

## Roles in this experiment

1. **Writer** (stage 1): research the code, write a fact sheet, then a cold draft. No handprint.
2. **Verifier** (stage 2, a different agent): checks every claim in the draft against the code.
3. **Reviser** (stage 3): fixes the verifier's findings, then runs the handprint loop.
4. **Judge** (stage 4, a different agent): blind pairwise comparison.

## Rules for every writer and reviser

- **Every factual claim comes from the code**, not from memory or plan files. The fact sheet lists
  each claim with its source as `path:line`. Status comes from code, tests and STATUS.md, never
  from plan/research files (they describe intent). If a feature is designed but not built, the
  doc must not present it as working.
- **UI labels are quoted exactly** as the display catalog has them
  (cadatomic: crates/cadatomic-ui/i18n/en.ftl). Keyboard shortcuts come from the keymap/inventory
  (cadatomic: docs/behavior/inventory.toml). Code identifiers are copied from source.
- **Read, don't build.** These workspaces are huge. Do not run cargo builds or tests. Reading,
  grep and git log are fine.
- **Write for a reader, not for the team.** No lane voice: no milestone ids, batch numbers, lane
  names, `§` references into plans, decision ids, or "the coordinator". No internal file paths in
  user-facing prose unless the reader needs them (a developer tutorial may name crate paths and
  files the reader edits).
- **Shape.** Tutorial: a learning path the reader follows start to finish and ends with something
  that works; each step says what to do and what they should see. How-to guide: a goal-first
  recipe for someone who already knows the basics; prerequisites, numbered steps (one action each),
  the result, and what to do when it goes wrong. Both: one recovery tip where a step commonly
  fails. Headings name the actual subject, not template words (no "Overview", "Introduction",
  "Conclusion", "Best Practices", "Summary"). No closing paragraph that restates the page.
- **Markdown**, 900–2,000 words of prose (code blocks extra). Code blocks only with code that
  exists or is copied from a real example/test; mark anything illustrative.
- **Mark gaps honestly.** If something the reader would need doesn't exist yet, say so in one plain
  sentence ("There is no installer yet; build from source").

## Output layout

`/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad/exp/<doc-id>/`
- `facts.md` — numbered claims `F1 …` with `path:line` sources (writer)
- `draft0.md` — the cold draft (writer); never edit it after stage 1
- `verify.md` — verifier report (verifier)
- `draft1.md` — after fixing verified errors (reviser)
- `draft2.md` … — handprint passes (reviser); `final.md` — the chosen version
- `log.md` — what each pass changed and why (reviser)
