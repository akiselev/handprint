#!/usr/bin/env python3
"""Make blind pairs for one doc: two runs, each with pair A (draft0 vs final) and pair B
(draft1 vs final), order swapped between runs. Key goes to judge-keys/."""
import sys, random, shutil, json
from pathlib import Path
S = Path("/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad")
doc = sys.argv[1]; src = S / "exp" / doc
rng = random.Random(doc)
key = {}
first = {"A": rng.random() < 0.5, "B": rng.random() < 0.5}  # True: final is X in run 1
for run in (1, 2):
    d = S / "judge" / doc / f"run{run}"; d.mkdir(parents=True, exist_ok=True)
    for pair, base in (("A", "draft0.md"), ("B", "draft1.md")):
        final_is_x = first[pair] if run == 1 else not first[pair]
        x, y = ("final.md", base) if final_is_x else (base, "final.md")
        shutil.copy(src / x, d / f"pair{pair}-X.md"); shutil.copy(src / y, d / f"pair{pair}-Y.md")
        key[f"run{run}/pair{pair}"] = {"X": x, "Y": y}
(S / "judge-keys" / f"{doc}.json").write_text(json.dumps(key, indent=1))
print(json.dumps(key))
