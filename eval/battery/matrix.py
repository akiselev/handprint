"""The coverage matrix, as data.

WIT.md states, per author, which feature families are signature axes (●●),
which are secondary (●), and which are present as a *low* band (○). The
interpretability suite asserts exactly that table, so the table lives here
rather than being restated in prose and in code.

Two things make this a real test rather than a restatement.

**The ○ cells are asserted too.** Bukowski is characterized by what he does not
do — adverbs, subordination, intensifiers — and a battery that only checked the
●● cells would pass on a reference that had learned "informal register" and
nothing else. Each author's `low` list must sit *below* the pooled-background
median, which is a claim a wrong reference fails.

**The dimension names are the shipped ones.** If a dimension is renamed, this
file breaks, which is the point: the coverage matrix is a promise about what
the crate measures, and a silent rename is a broken promise.
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class AuthorAxes:
    """What a reference fitted on this author must show."""

    #: Feature families to fit. Names are `handprint fit --features` values.
    features: list[str]
    #: Dimensions that must appear among the author's top deviations.
    #: The assertion is set *intersection*, not equality — an author has more
    #: signal than any list captures, and demanding equality would make the
    #: test a snapshot rather than a claim.
    high: list[str]
    #: Dimensions that must sit *below* the background median. Absence is a
    #: fingerprint, and only a two-sided band can express it.
    low: list[str] = field(default_factory=list)
    #: How many of `high` must be present. Defaults to all but one, because a
    #: single dimension missing from a top-k list is noise and the claim is
    #: about the profile rather than about any one number.
    min_high: int | None = None

    def required(self) -> int:
        if self.min_high is not None:
            return self.min_high
        return max(1, len(self.high) - 1)


#: Every family the author battery fits. One pipeline for all five authors, so
#: that a dimension present for one and absent for another is a fact about the
#: author rather than about the configuration.
BATTERY_FEATURES = [
    "punct",
    "sentence-rhythm",
    "punchline",
    "biber",
    "readability",
    "frames",
    "formality",
    "devices",
    "syntax",
    "verse",
    "profanity",
    "hyland",
    "ngrams",
]

AUTHORS: dict[str, AuthorAxes] = {
    "douglas-adams": AuthorAxes(
        features=BATTERY_FEATURES,
        high=[
            "frame.negated_vehicle_rate",
            "form.clash_rate",
            "surp.punch_spike_rate",
            "dev.litotes_rate",
        ],
        low=["syn.fragment_rate"],
    ),
    "terry-pratchett": AuthorAxes(
        features=BATTERY_FEATURES,
        high=[
            "surp.punch_spike_rate",
            "form.clash_rate",
            # Either reading of the absurd-precision axis counts; the matrix
            # names them together.
            "dev.litotes_rate",
            "dev.absurd_precision_rate",
            # The digression axis. `md.footnote_rate` only where the source
            # formatting preserved footnotes, so the aside rate leads.
            "sent.aside_rate",
        ],
        min_high=3,
    ),
    "charles-bukowski": AuthorAxes(
        features=BATTERY_FEATURES,
        high=[
            "syn.fragment_rate",
            "lex.profanity.cat.strong",
            "syn.concreteness_mean",
            "verse.one_word_line_rate",
        ],
        # The ○ cells. This is the half of the matrix a naive imitator fails.
        low=[
            "syn.ly_adverb_rate",
            "syn.sub_coord_ratio",
            "syn.intensifier_rate",
        ],
        min_high=2,
    ),
    "hunter-s-thompson": AuthorAxes(
        features=BATTERY_FEATURES,
        high=[
            "syn.booster_chain_max",
            "syn.allcaps_exclaim_rate",
            "punct.ellipsis_rate",
            "contrast.hst.w1",
        ],
        min_high=2,
    ),
    "shakespeare": AuthorAxes(
        features=BATTERY_FEATURES,
        high=[
            "verse.feminine_ending_rate",
            "verse.stress_pos",
            "verse.thou_you_ratio",
        ],
        min_high=2,
    ),
}


#: The register battery's claims, in the same shape.
REGISTERS: dict[str, AuthorAxes] = {
    "tech-blog": AuthorAxes(
        features=BATTERY_FEATURES,
        high=[
            "sent.aside_rate",
            "md.link_rate",
            "md.footnote_rate",
            "sent.rhetorical_question_rate",
            "lex.hyland.cat.hedges",
        ],
        min_high=3,
    ),
    "marketing": AuthorAxes(
        features=BATTERY_FEATURES + ["marketing"],
        high=[
            "dev.imperative_rate",
            "lex.marketing-eval.cat.pressure",
            "lex.marketing-eval.cat.directive",
            "dev.headline",
        ],
        min_high=2,
    ),
    "agent-prose": AuthorAxes(
        features=BATTERY_FEATURES,
        # The PNAS LLM signature: more nominalizations and participial clauses,
        # fewer contractions and first-person pronouns.
        high=[
            "biber.nominalization_rate",
            "biber.participial_clause_rate",
        ],
        low=["punct.contraction_rate", "biber.pron.p1"],
        min_high=1,
    ),
}


def matches(dimension: str, claims: list[str]) -> bool:
    """Whether a dimension id satisfies one of a matrix cell's claims.

    Prefix matching, because several claims name a *family* of dimensions —
    `verse.stress_pos` covers ten of them, `dev.headline` four — and spelling
    out every member would make the matrix a list of dimension names rather
    than a statement about axes.
    """
    return any(dimension == c or dimension.startswith(c + ".") for c in claims)
