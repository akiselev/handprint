# Comic-move labelling prompt (v1, 2026-08-01)

You are labelling passages from one author's prose so that a later retrieval
step can find them **by technique rather than by subject**. You are not judging
quality and you are not summarizing.

## Input

One passage, 100–400 words, from a single author.

## Output

JSON matching `schema.json`:

```json
{
  "devices": ["subverted_simile", "register_clash"],
  "scene_function": "character_intro",
  "evidence": {"subverted_simile": "in much the same way that bricks don't"},
  "confidence": "high"
}
```

## Device vocabulary

Use only these. If a passage uses something not on the list, use `other` and
say what in `evidence.other`. A vocabulary that grows during a labelling run
produces an index nobody can query.

| label | what it is |
|---|---|
| `subverted_simile` | a comparison whose vehicle undercuts the comparison |
| `register_clash` | two registers in one sentence — bureaucratic for cosmic, slang in a formal frame |
| `absurd_precision` | a precise measurement of something unmeasurable |
| `litotes` | assertion by negated understatement |
| `transferred_epithet` | a mental-state adjective on an object |
| `digression` | the narrator leaves the scene and comes back |
| `deadpan_escalation` | a sequence that grows without the prose acknowledging it |
| `bathos` | a descent from the elevated to the trivial, in that order |
| `catalogue` | a list whose length is the joke |
| `direct_address` | the narrator speaks to the reader |
| `hyperbole` | overt exaggeration, not ironic |
| `understated_violence` | something terrible reported flatly |
| `pedantic_correction` | a clarification nobody asked for |
| `false_syllogism` | a chain of reasoning that is wrong in a specific way |
| `none` | the passage is not doing anything comic |
| `other` | something real that is not on this list |

## Scene functions

`character_intro`, `technology_failing`, `narratorial_aside`, `dialogue_scene`,
`description_of_place`, `exposition`, `action`, `arrival`, `departure`,
`argument`, `reflection`, `transition`.

## Rules

1. **Point at the words.** Every device you name needs an `evidence` span
   copied from the passage. A device you cannot quote is a device you inferred
   from the vibe.
2. **At most three devices.** A passage that seems to use six is a passage you
   are pattern-matching rather than reading.
3. **`none` is a real answer** and should be common. Most of any author's prose
   is not doing a bit. An index where every chunk is comic is an index that
   cannot rank.
4. **Do not label quality.** "This one works" is the wit critic's job, and
   mixing the two makes the retrieval prefer the labeller's taste.
5. **`confidence: low`** when you are guessing. Low-confidence labels are kept
   and are excluded from the κ sample, which is what keeps the agreement
   number honest.

## What this is for

The retrieval step will ask for "three passages where this author introduces a
character using register clash". If your labels are accurate, that query
returns three usable exemplars. If they are aspirational, it returns three
passages that share a label and nothing else, and the rewrite that follows will
be worse than one with no exemplars at all.
