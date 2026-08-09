# The transfer matrix — 30 author-to-author cells

One transfer tells you a model can write a pastiche. A matrix tells you *which
kinds of distance* a calibrated critic can actually close, which is the only
question the objective function exists to answer.

Poe → Pratchett (`../adams-pratchett/STORY-RESULTS.md`) turned out to be near
the hard end, and not for the reason it looks: the gap is not gothic-versus-
comic, it is that a first-person interior monologue with no dialogue cannot
reach a third-person dialogue-heavy voice without inventing scenes. Two of its
dimensions were **content-bound**, not style-bound. That distinction is what
this matrix is built to map.

## Axes

Every cell names the dominant shift it exercises. A matrix of random pairs
would be a list of anecdotes.

| axis | pole A | pole B |
|---|---|---|
| **P** person / stance | 1st-person interior | 3rd-person omniscient |
| **R** register height | latinate, periodic, hypotactic | plain, Anglo-Saxon, paratactic |
| **W** wit mechanism | earnest · irony · epigram · absurdism · bathos · vernacular tall-tale |
| **F** form | narrative · dialogue · essay · epic verse · folktale |
| **E** era diction | pre-1850 | 20th century |

## Design rules

1. **Both directions on every axis.** Injecting a device and removing one are
   different machinery — the capstone predicts over-firing on injection and a
   mechanics-only pastiche on removal. Cells are paired so each has a mirror.
2. **Comic → comic cells are included and are the hard ones.** Twain to Wilde
   is a small distance on genre and a large one on wit mechanism. If the critic
   cannot separate two comic authors it is measuring formality.
3. **Controls at both ends.** A near-neighbour floor (the Brontë sisters) and a
   maximum-distance ceiling (Milton's blank verse to comic prose).
4. **One render per cell.** The showcase transfer got five critique iterations;
   these get one render plus one revision. The matrix question is *which
   shifts are hard*, not how far any single cell can be pushed. Cells are
   therefore comparable to each other and **not** to the showcase.

## The cells

Wodehouse was in an earlier draft of this matrix and has been removed. The
reason is recorded because it is a methodological point, not a taste one: the
representative-passage check below put his sampled passage closer to Twain's
reference than to his own, and re-sampling showed why — his narratorial
chapter-openings are mock-philosophical essay, a genre Twain owns, while his
dialogue is a different instrument entirely. An author whose own corpus splits
into two registers that far apart is a bad *target*, because "sounds like
Wodehouse" is not one thing.

### Comic injection — serious source, comic target (tests device injection)

| id | source → target | axis | named shift |
|---|---|---|---|
| C01 | Poe → Pratchett | P R W | gothic maximalism → bathos *(showcase, 5 iterations)* |
| C02 | Poe → Twain | R W | gothic → American vernacular |
| C03 | Dostoyevsky → Adams | P W | existential interior → absurdist cosmic |
| C04 | Melville → Jerome | R W | high rhetoric → whimsical digression |
| C05 | Conan Doyle → Pratchett | W | deductive procedural → comic fantasy |
| C06 | E. Brontë → Adams | P W | gothic romance → absurdist |
| C07 | Tolstoy → Twain | P R W | realist omniscient → vernacular first person |
| C08 | Plato → Adams | F W | philosophical dialogue → absurd SF dialogue |
| C09 | Homer → Pratchett | F R W | epic register → comic prose |
| C10 | Thoreau → Jerome | F W | earnest essay → comic essay |

### Comic removal — comic source, serious target (tests absence bands)

| id | source → target | axis | named shift |
|---|---|---|---|
| C11 | Twain → Poe | R W | vernacular → gothic |
| C12 | Adams → Melville | R W E | absurdism → cetological high rhetoric |
| C13 | Pratchett → Dostoyevsky | W | bathos → existential earnestness |
| C14 | Twain → Austen | P R W | vernacular → ironic formality |
| C15 | Jerome → Conan Doyle | W | whimsy → procedural |

### Mechanism swap — comic to comic (the subtle cells)

| id | source → target | axis | named shift |
|---|---|---|---|
| C16 | Dickens → Twain | W E | Victorian ornament → American vernacular |
| C17 | Adams → Austen | W E | absurdism → ironic formality |
| C18 | Twain → Wilde | R W | vernacular → epigram |
| C19 | Pratchett → Adams | W | bathos → absurdism *(near neighbour)* |
| C20 | Jerome → Pratchett | W E | Victorian whimsy → modern comic fantasy |
| C21 | Wilde → Pratchett | W | epigram → bathos |

### Serious to serious — register, person and form controls

| id | source → target | axis | named shift |
|---|---|---|---|
| C22 | Austen → Dickens | R W | ironic free-indirect → grotesque catalogue |
| C23 | Wells → Poe | R | plain speculative → gothic maximal |
| C24 | Grimm → Dickens | R F | folktale plainness → Victorian ornament |
| C25 | Joyce → Grimm | R F | stream of consciousness → folktale plainness |
| C26 | Machiavelli → Wilde | F W | political treatise → epigram |

### Verse — the form axis the prose corpus cannot reach

| id | source → target | axis | named shift |
|---|---|---|---|
| C27 | Milton → Pratchett | F R | **epic blank verse → comic prose** |
| C28 | Whitman → Austen | F R | free verse → ironic formal prose |

### Controls

| id | source → target | expectation |
|---|---|---|
| C29 | Adams → Pratchett | near neighbour; small movement needed |
| C30 | E. Brontë → C. Brontë | **floor control** — siblings, same decade, same genre |

## Two gates before a cell counts

**The source must belong to its source author.** Every source passage is
scored against all 22 references and must rank its own author first. This
caught the Wodehouse problem and would have let a mislabelled cell through
otherwise. 13 of 14 sampled passages passed on the first attempt.

**The output must convince a reader, not just the metric.** `p_same_author`
rising is necessary and nowhere near sufficient — a rewrite can move toward a
reference by matching punctuation and sentence length while reading as
nothing at all. Every cell is therefore also put to a **blind LLM judge** under
`../../judge/rubric.md`: the judge is shown the rewrite and a genuine passage
by the target author, unlabelled and in randomised order, and must say which is
genuine, score the imitation's fidelity, and name its failures with quotations.
The judge is a subagent with no access to this conversation, so it does not
know which side it is being asked to defend.

The headline number is therefore **not** the distance. It is the judge's
discrimination rate: if a blind reader picks the real author every time, the
transfer failed however far the metric moved.

## Method

Source passages are ~1,000 words. That is below the 2,000-word floor for
whole-profile attribution, so **`p_same_author` is reported as indicative and
the per-family findings are the evidence** — which is what `LengthTier::Unreliable`
tells you to do. At this length `punct`, `sentence`, `syntax`, `register`,
`rhythm` and `char_ngram` are all inside their usable range; `mfw` is not, and
`device` is usable but not supported. The trade is deliberate: 28 comparable
cells at reduced per-cell power, rather than four at full power. The blind
judge does not care about token counts, which is a further reason not to let
the distance be the headline.

Every reference is fitted **without the contrast family**, per the finding in
`../adams-pratchett/RESULTS.md` that it reports topic rather than style. All
are calibrated against the same 26-author background, so cells are comparable.
