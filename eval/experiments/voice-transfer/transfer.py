#!/usr/bin/env python3
"""Score one transfer arm against the five capstone criteria.

The rewriting itself is an agent's job — the `voice-transfer` skill is the
procedure, and this script does not embed a model. What it does is the part
that has to be mechanical: read the final draft, the plain-retelling baseline
and the two references, and say whether each criterion was met.

All five, or the transfer is a falsification. Writing up a four-of-five as a
success is the failure this whole design exists to prevent, so `passed` is a
conjunction and the per-criterion detail is always printed.

    python3 transfer.py --build build/voice-transfer --transfer T1 --arm c
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "battery"))

from hp import Handprint  # noqa: E402

#: Which reference each transfer moves toward and away from.
TRANSFERS = {
    "T1": {"from": "wodehouse", "to": "hemingway-early", "source": "extricating-young-gussie"},
    "T2": {"from": "hemingway-early", "to": "wodehouse", "source": "cat-in-the-rain"},
    "T3": {"from": "twain", "to": "poe", "source": "jumping-frog"},
}

#: Families whose findings the anti-caricature criterion covers, in **both**
#: directions. Over-firing is caricature; under-firing is a mechanics-only
#: pastiche that never acquired the voice.
DEVICE_FAMILIES = {"device", "syntax", "rhythm"}
DEVICE_PREFIXES = ("dev.", "frame.", "syn.", "surp.punch")

#: The Balanced threshold. Strict is reported alongside, never instead.
BALANCED = 0.10
STRICT = 0.25


def criterion_gate(report: dict) -> dict:
    p = report["verdict"].get("p_same_author")
    return {
        "p_same_author": p,
        "balanced": p is not None and p >= BALANCED,
        "strict": p is not None and p >= STRICT,
        "passed": p is not None and p >= BALANCED,
    }


def criterion_away(
    hp: Handprint, from_ref: Path, plain: Path, final: Path, exemplar: Path
) -> dict:
    """Distance to the source author must strictly increase."""
    before = hp.compare(from_ref, plain, exemplar)["distance"]
    after = hp.compare(from_ref, final, exemplar)["distance"]
    return {
        "distance_before": before,
        "distance_after": after,
        "passed": after > before,
    }


def criterion_anticaricature(report: dict) -> dict:
    """Zero Medium-or-above findings **in either direction** on the device
    families. Inside the band, not merely below the ceiling."""
    offenders = [
        {
            "id": f["id"],
            "direction": f["direction"],
            "severity": f["severity"],
            "observed": f["observed"],
            "band": f["target_band"],
        }
        for f in report["findings"]
        if f["severity"] in {"medium", "high"}
        and (f["family"] in DEVICE_FAMILIES or f["id"].startswith(DEVICE_PREFIXES))
    ]
    return {
        "offenders": offenders,
        "over_fired": [o for o in offenders if o["direction"] == "reduce"],
        "under_fired": [o for o in offenders if o["direction"] == "increase"],
        "passed": not offenders,
    }


def criterion_content(reports: list[dict]) -> dict:
    overlaps = [r["guards"].get("content_overlap") for r in reports]
    drift = all(r["guards"]["drift_ok"] for r in reports)
    canary = all(r["guards"]["canary_ok"] for r in reports)
    return {
        "content_overlap": [o for o in overlaps if o is not None],
        "drift_ok": drift,
        "canary_ok": canary,
        "passed": drift and canary,
    }


def criterion_judge(judge_path: Path | None, arm: str) -> dict:
    """Arm ordering from the wit critic, when one has been run."""
    if judge_path is None or not judge_path.exists():
        return {
            "passed": None,
            "note": "no wit-critic run; criterion 5 is unevaluated, not met",
        }
    data = json.loads(judge_path.read_text(encoding="utf-8"))
    rates = {a: r.get("win_rate") for a, r in data.get("arms", {}).items()}
    ordered = (
        rates.get("c") is not None
        and rates.get("b") is not None
        and rates.get("a") is not None
        and rates["c"] > rates["b"] > rates["a"]
    )
    return {
        "win_rates": rates,
        "arm": arm,
        "ordering_holds": ordered,
        # The claim is the ordering. The absolute rate against the real author
        # is reported and is expected below 0.5.
        "absolute_vs_gold": rates.get(arm),
        "passed": ordered,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--transfer", choices=sorted(TRANSFERS), required=True)
    parser.add_argument("--arm", choices=["a", "b", "c"], default="c")
    parser.add_argument("--handprint", default="handprint")
    parser.add_argument("--judge", type=Path, default=None)
    args = parser.parse_args()

    spec = TRANSFERS[args.transfer]
    hp = Handprint(command=args.handprint)
    artifacts = args.build / "artifacts" / f"{args.transfer}-{args.arm}"
    plain = artifacts / "plain.md"
    final = artifacts / "final.md"
    reports_path = artifacts / "reports.json"
    for path in (plain, final, reports_path):
        if not path.exists():
            raise SystemExit(
                f"{path} is missing. Run the transfer first — the "
                f"`voice-transfer` skill produces plain.md, final.md and the "
                f"per-iteration reports."
            )

    to_ref = args.build / "refs" / f"{spec['to']}.json"
    from_ref = args.build / "refs" / f"{spec['from']}.json"
    for path in (to_ref, from_ref):
        if not path.exists():
            raise SystemExit(f"{path} is missing; run run.sh first")
    # The reports must have been produced against the toward-reference, or
    # criteria 1, 3 and 4 are reading someone else's numbers.
    expected = json.loads(to_ref.read_text(encoding="utf-8"))["provenance"]["name"]
    exemplars = sorted((args.build / "corpora" / spec["from"]).glob("*.md"))
    if not exemplars:
        raise SystemExit(f"no exemplars for {spec['from']}; run run.sh first")

    reports = json.loads(reports_path.read_text(encoding="utf-8"))
    final_report = reports[-1]
    actual = final_report["reference"]["name"]
    if actual != expected:
        raise SystemExit(
            f"the critique reports were produced against {actual!r}, not "
            f"{expected!r}. Criteria 1, 3 and 4 would be reading another "
            f"author's bands."
        )

    result = {
        "transfer": args.transfer,
        "arm": args.arm,
        "from": spec["from"],
        "to": spec["to"],
        "criteria": {
            "1_gate": criterion_gate(final_report),
            "2_away": criterion_away(hp, from_ref, plain, final, exemplars[0]),
            "3_anticaricature": criterion_anticaricature(final_report),
            "4_content": criterion_content(reports),
            "5_judge": criterion_judge(args.judge, args.arm),
        },
    }
    met = [k for k, v in result["criteria"].items() if v["passed"] is True]
    unevaluated = [k for k, v in result["criteria"].items() if v["passed"] is None]
    result["passed"] = len(met) == len(result["criteria"])
    result["met"] = met
    result["unevaluated"] = unevaluated

    out = artifacts / "criteria.json"
    out.write_text(json.dumps(result, indent=2), encoding="utf-8")

    for name, value in result["criteria"].items():
        state = {True: "PASS", False: "FAIL", None: "UNEVALUATED"}[value["passed"]]
        print(f"{state:12} {name}")
    print(f"\n{len(met)}/{len(result['criteria'])} criteria met -> {out}")
    if not result["passed"]:
        print(
            "This transfer did not meet every criterion. Write it up as a "
            "falsification finding; a four-of-five reported as a success is "
            "the failure this design exists to prevent."
        )
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
