#!/usr/bin/env python3
"""Summarize a critique report: the gate, the guards, and findings by family."""
import json, sys, collections
d = json.load(open(sys.argv[1]))
v, g, doc = d['verdict'], d['guards'], d['doc']
print(f"tokens {doc['tokens']}  tier {doc['length_tier']}  imputed {doc.get('imputed_dims')}")
low = [k for k, val in doc['confidence'].items() if val in ('low', 'none')]
print(f"low-confidence families: {low or 'none'}")
print(f"GATE pass={v.get('pass')} p_same_author={v.get('p_same_author')} "
      f"p_vs_unrelated={v.get('p_value_vs_unrelated')} distance={v.get('distance'):.4f}")
print(f"guards canary_ok={g['canary_ok']} drift_ok={g['drift_ok']} "
      f"overlap={g.get('content_overlap')}")
print(f"findings_total={d['findings_total']}")
fs = d['findings']
sev = collections.Counter(f['severity'] for f in fs)
print("severity:", dict(sev))
# The skill's gate condition 3 is about these four families specifically.
GATED = {'device', 'frames', 'syntax', 'rhythm'}
by = collections.defaultdict(list)
for f in fs:
    by[f['family']].append(f)
print("\nby family (medium+ count / total):")
for fam in sorted(by):
    med = [f for f in by[fam] if f['severity'] in ('medium', 'high')]
    mark = ' <-- GATED' if fam in GATED else ''
    print(f"  {fam:12s} {len(med):3d} / {len(by[fam]):3d}{mark}")
print("\ngated-family medium+ findings:")
for f in fs:
    if f['family'] in GATED and f['severity'] in ('medium', 'high'):
        b = f['target_band']
        print(f"  {f['severity']:6s} {f['id']:42s} {f['direction']:8s} "
              f"obs={f['observed']:.2f} band=[{b[0]:.2f},{b[1]:.2f}] z={f['z']:+.1f}")
print("\ntop 20 by |z|:")
for f in sorted(fs, key=lambda x: -abs(x['z']))[:20]:
    b = f['target_band']
    print(f"  {f['severity']:6s} {f['id']:42s} {f['direction']:8s} "
          f"obs={f['observed']:.2f} band=[{b[0]:.2f},{b[1]:.2f}] z={f['z']:+.1f}")
