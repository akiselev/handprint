# The capstone: cross-author voice transfer on public-domain corpora

The plan's exit criterion, and deliberately double-duty. Every corpus here is
public domain in the US under the 95-year rule, so this experiment produces
handprint's shippable demo packs and README numbers — which the in-copyright
battery authors can never provide.

## The jurisdiction rule

US public domain is not worldwide public domain. Wodehouse (d. 1975) and
Hemingway (d. 1961) remain in copyright in life+70 jurisdictions until 2046 and
2032. So the shippables split by whether they **carry text**:

| artifact | Wodehouse, Hemingway | Twain, Poe, Austen |
|---|---|---|
| fitted reference, calibration, band edges | ship, with `pd_basis` stated | ship |
| metrics, finding counts, short quotations | ship | ship |
| move-index sidecars (quoted passages) | local only | ship |
| full transfer transcripts (embed the source) | local only | ship |

`handprint_data::gutenberg::capstone_authors()` holds this as data so the
scripts and the provenance read the same claim. `eval/licenses/check.py` fails
the build if a local-only artifact is ever tracked.

## Authors

| author | PD basis | pole exercised |
|---|---|---|
| P. G. Wodehouse | US: pre-1930 works | comic-device pole: transferred epithet, simile frames, slang-in-formal-frame clash, punch rhythm |
| early Hemingway | US: 1925–1929 works | minimalism pole: fragments, parataxis, adverb scarcity, low device rates |
| Mark Twain | worldwide | vernacular + comic digression: dialect orthography, 1st-person deixis, aside rate |
| E. A. Poe | worldwide | gothic maximalism: adjective and arousal density, hypotaxis, em-dash exuberance |
| Jane Austen (stretch) | worldwide | irony + hypotaxis, litotes pole |

Wodehouse↔Hemingway is the designed-in headline pair: maximum axis separation,
and the two directions test different machinery.

## Transfers

**T1 (comic → minimal)** — Wodehouse, "Extricating Young Gussie" (1915), into
Hemingway's voice. Tests the *absence* bands. Device, frame, adverb and
subordination dimensions must descend **into** Hemingway's low bands. "Write
minimally" gives an agent no target rate; a calibrated critic does.

**T2 (minimal → comic)** — Hemingway, "Cat in the Rain" (*In Our Time*, 1925),
into Wodehouse's voice. Tests calibrated device *injection*. The documented
failure mode is over-firing, so this is the live anti-caricature trial:
upper-band `Reduce` findings have to steer the agent back down.

**T3 (stretch, vernacular → maximal)** — Twain, "The Celebrated Jumping Frog"
(1865), into Poe's voice. The largest orthography and punctuation shift, and —
both authors being worldwide-PD — the transfer whose full transcript is
committable, which makes it the published-transcript exemplar even if T1 and T2
carry the README numbers.

## Arms

Each transfer runs three, so the claim is about the pipeline rather than about
the model:

* **(a) zero-shot** — "rewrite in B's voice", nothing else.
* **(b) critique loop, mechanics only** — arm (a) plus `handprint critique`
  iterations acting on punctuation, sentence and frequency findings.
* **(c) full pipeline** — the `voice-transfer` skill: plain retelling → device
  plan at B-calibrated rates → render with move-matched exemplars → dual-critic
  polish until the gate or a cap of 12.

The plain retelling from stage 1 is the draft the `Critic` is seeded with, so
`content_overlap` is measured against it.

## Success criteria, per transfer, arm (c)

All five, or the transfer is a falsification:

1. **Gate** — `p_same_author` against reference B at or above the Balanced
   threshold (0.10); Strict (0.25) reported too.
2. **Away** — Burrows distance to reference A's exemplars strictly increases
   against the plain-retelling baseline, and GI `verify` with the other
   capstone authors as impostors returns `TargetFavoured` for B and not for A.
3. **Anti-caricature** — zero Medium-or-above findings **in either direction**
   on Device, Frames, Syntax and Rhythm. Inside the band, not merely below the
   ceiling: over-firing is caricature and under-firing is a mechanics-only
   pastiche that never acquired the voice.
4. **Content** — `guards.drift_ok` and `guards.canary_ok` true throughout.
5. **External judge** — blind pairwise wit-critic against gold B anchors: arm
   (c) beats (b) beats (a), with the judge blinded to arm. The absolute win
   rate against real B is reported honestly and is expected below 0.5. The
   claim is the *ordering*.

## Running it

```sh
./run.sh            # corpora -> references -> transfers -> manifest
```

`run.sh` regenerates everything from Gutenberg IDs and seeds. The manifest
records model ids, seeds, pack versions and the handprint version, because a
number without those is not reproducible and should not be quoted.

## This experiment is allowed to fail

That is its job. If T1 or T2 misses a criterion, the write-up says which and
why. A capstone that can only succeed was not a test.
