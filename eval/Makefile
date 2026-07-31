# Eval batteries.
#
# Every target here is re-runnable from a clean checkout *given the corpora*.
# Corpora are not in the checkout and never will be: the in-copyright ones
# (Adams, Pratchett, Bukowski, Thompson, Folger Shakespeare) are local-only,
# and the public-domain ones are fetched by `make corpora`. What lands in git
# is scripts, synthetic fixtures, and metrics.

HANDPRINT ?= cargo run --quiet --manifest-path ../Cargo.toml --release --bin handprint --
PYTHON    ?= python3
CORPORA   ?= corpora
BUILD     ?= build

.PHONY: help battery authors registers licenses corpora gutenberg clean fixtures

help:
	@echo "make licenses   license-hygiene check (runs in CI, needs no corpora)"
	@echo "make fixtures   regenerate the seeded anti-caricature pastiches"
	@echo "make gutenberg  prepare the public-domain capstone corpora"
	@echo "make authors    author battery: separation, interpretability, anti-caricature"
	@echo "make registers  register battery: blog vs marketing vs agent prose"
	@echo "make battery    authors + registers, writing RESULTS.md"
	@echo ""
	@echo "Corpora live in $(CORPORA)/{author}/ as per-chapter .md files."
	@echo "In-copyright corpora are local-only and are never committed."

# The only target that needs nothing but the repo. CI runs this one.
licenses:
	$(PYTHON) licenses/check.py

fixtures:
	$(PYTHON) battery/fixtures.py --out $(BUILD)/fixtures

# Public-domain corpora for the capstone. Requires raw/ to hold
# raw/{author}/*.txt downloaded from Project Gutenberg by ID.
gutenberg:
	$(HANDPRINT) gutenberg --in raw --out $(CORPORA) --jsonl $(BUILD)/gutenberg.jsonl

authors: fixtures
	$(PYTHON) battery/run.py --corpora $(CORPORA) --build $(BUILD) \
		--handprint "$(HANDPRINT)"

registers:
	$(PYTHON) registers/run.py --corpora $(CORPORA) --build $(BUILD) \
		--handprint "$(HANDPRINT)"

battery: authors registers
	$(PYTHON) battery/results.py --build $(BUILD) --out RESULTS.md

clean:
	rm -rf $(BUILD)
