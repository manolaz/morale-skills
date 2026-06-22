"""Analyze pgdrsa_results.jsonl vs RSA paper Table 1.

Two verdict mappings (per owner direction, no LLM judge):
  pgdrsa-strict: malicious = {executed_contained}
  pgdrsa-loose:  malicious = {executed_contained, attempted_blocked}

Both compared to RSA's 88.0% TPR / 8.0% FPR / 90.0% Acc.

Usage: python -m pgdrsa.analyze [results.jsonl]
"""
from __future__ import annotations
import json, sys
from pathlib import Path

RESULTS = Path(sys.argv[1] if len(sys.argv) > 1 else "pgdrsa_results.jsonl")

RSA = {"tpr": 0.880, "fpr": 0.080, "acc": 0.900}


def _load() -> list[dict]:
    out = []
    for line in RESULTS.read_text().splitlines():
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            pass
    return out


def _metrics(records: list[dict], malicious_pred) -> dict:
    """records = list of {label, verdict, finalization_status}. Returns TPR/FPR/Acc + counts."""
    tp = fp = tn = fn = invalid = 0
    for r in records:
        if r.get("finalization_status") == "INVALID" or r.get("error"):
            invalid += 1
            continue
        actual_mal = r["label"] == "malicious"
        pred_mal = malicious_pred(r.get("verdict"))
        if actual_mal and pred_mal: tp += 1
        elif actual_mal and not pred_mal: fn += 1
        elif (not actual_mal) and pred_mal: fp += 1
        else: tn += 1
    n_mal = tp + fn
    n_ben = tn + fp
    tpr = tp / n_mal if n_mal else 0.0
    fpr = fp / n_ben if n_ben else 0.0
    acc = (tp + tn) / (n_mal + n_ben) if (n_mal + n_ben) else 0.0
    return {"tp": tp, "fp": fp, "tn": tn, "fn": fn, "invalid": invalid,
            "tpr": tpr, "fpr": fpr, "acc": acc, "n_mal": n_mal, "n_ben": n_ben}


def _row(name: str, m: dict) -> str:
    return (f"| {name} | {m['tp']}/{m['n_mal']}={m['tpr']*100:5.1f}% | "
            f"{m['fp']}/{m['n_ben']}={m['fpr']*100:5.1f}% | "
            f"{m['acc']*100:5.1f}% | {m['invalid']} |")


def main() -> int:
    recs = _load()
    done = [r for r in recs if not r.get("error")]
    errored = [r for r in recs if r.get("error")]
    print(f"# pgdrsa vs RSA paper — comparison report\n")
    print(f"Records: {len(recs)} total | {len(done)} usable | {len(errored)} errored\n")

    if not done:
        print("No usable records yet."); return 1

    strict = _metrics(done, lambda v: v == "executed_contained")
    loose = _metrics(done, lambda v: v in ("executed_contained", "attempted_blocked"))

    print("## Headline (vs RSA Table 1)\n")
    print("| Method | TPR | FPR | Acc | Invalid |")
    print("|---|---|---|---|---|")
    print(f"| **RSA (paper)** | {RSA['tpr']*100:5.1f}% | {RSA['fpr']*100:5.1f}% | {RSA['acc']*100:5.1f}% | 0 |")
    print(_row("pgdrsa-strict", strict))
    print(_row("pgdrsa-loose", loose))

    print(f"\n## Confusion matrices\n")
    for name, m in [("pgdrsa-strict", strict), ("pgdrsa-loose", loose)]:
        print(f"### {name}  (TP={m['tp']} FP={m['fp']} TN={m['tn']} FN={m['fn']} invalid={m['invalid']})\n")
        print("| | pred malicious | pred benign | total |")
        print("|---|---|---|---|")
        print(f"| **actual malicious** | {m['tp']} | {m['fn']} | {m['n_mal']} |")
        print(f"| **actual benign**    | {m['fp']} | {m['tn']} | {m['n_ben']} |\n")

    print("## Verdict distribution\n")
    by_label_verdict: dict[tuple[str, str], int] = {}
    for r in done:
        key = (r["label"], r.get("verdict", "?"))
        by_label_verdict[key] = by_label_verdict.get(key, 0) + 1
    print("| label | verdict | count |")
    print("|---|---|---|")
    for (lab, ver), n in sorted(by_label_verdict.items()):
        print(f"| {lab} | {ver} | {n} |")

    if errored:
        print(f"\n## Errored runs ({len(errored)})\n")
        for r in errored[:20]:
            print(f"- `{r['skill_name']}` ({r['label']}): {r['error']}")
        if len(errored) > 20:
            print(f"... and {len(errored)-20} more")

    # Misclassifications (strict)
    print(f"\n## pgdrsa-strict misclassifications\n")
    print("### False negatives (malicious, predicted benign)\n")
    fns = [r for r in done if r["label"] == "malicious" and r.get("verdict") != "executed_contained"]
    for r in fns:
        print(f"- `{r['skill_name']}`: verdict={r.get('verdict')} status={r.get('finalization_status')} "
              f"hits={r.get('receiver_hits')} time={r.get('elapsed_s')}s")
    print("\n### False positives (benign, predicted malicious)\n")
    fps = [r for r in done if r["label"] == "benign" and r.get("verdict") == "executed_contained"]
    for r in fps:
        print(f"- `{r['skill_name']}`: verdict={r.get('verdict')} hits={r.get('receiver_hits')}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
