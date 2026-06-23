from __future__ import annotations

import argparse
import json
import os
import random
import sys
import time
import traceback
from pathlib import Path
from typing import Any

PACKAGE_ROOT = Path(__file__).resolve().parents[1]
if str(PACKAGE_ROOT) not in sys.path:
    sys.path.insert(0, str(PACKAGE_ROOT))

from pgdrsa import config as _config
from pgdrsa.smoke import run_smoke


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _skills_root() -> Path:
    return _repo_root() / "morale-rust" / "data" / "skills"


def _enumerate_skills() -> list[tuple[str, Path]]:
    root = _skills_root()
    if not root.exists():
        raise FileNotFoundError(f"skills root not found: {root}")

    skills: list[tuple[str, Path]] = []
    for label in ("benign", "malicious"):
        label_root = root / label
        if not label_root.exists():
            continue
        for skill_dir in sorted(label_root.iterdir()):
            if skill_dir.is_dir() and (skill_dir / "SKILL.md").exists():
                skills.append((label, skill_dir))
    return skills


def _configure_image(image: str | None) -> None:
    if not image:
        return
    os.environ["PGDRSA_IMAGE_REF"] = image
    _config.IMAGE_REF = image
    from importlib import reload
    from pgdrsa import smoke

    reload(smoke)
    globals()["run_smoke"] = smoke.run_smoke


def _pick_random_skills(skills: list[tuple[str, Path]], count: int, seed: int | None) -> list[tuple[str, Path]]:
    if count <= 0:
        raise ValueError("count must be > 0")
    if count >= len(skills):
        return list(skills)
    rng = random.Random(seed)
    return rng.sample(skills, count)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run a random sample of pgdrsa smoke benchmarks.")
    parser.add_argument("--count", type=int, default=10, help="number of random skills to run")
    parser.add_argument("--seed", type=int, default=None, help="optional random seed for reproducibility")
    parser.add_argument("--results-file", default="pgdrsa_random_results.jsonl",
                        help="JSONL output path (default: pgdrsa_random_results.jsonl)")
    parser.add_argument("--image", help="override PGDRSA_IMAGE_REF for this run")
    args = parser.parse_args()

    _configure_image(args.image)

    skills = _enumerate_skills()
    todo = _pick_random_skills(skills, args.count, args.seed)

    out_path = Path(args.results_file)
    if out_path.parent != Path(""):
        out_path.parent.mkdir(parents=True, exist_ok=True)
    mode = "a" if out_path.exists() else "w"

    print(f"Total skills available: {len(skills)}", flush=True)
    print(f"Random sample size: {len(todo)}", flush=True)
    if args.seed is not None:
        print(f"Seed: {args.seed}", flush=True)
    print("Selected skills:", flush=True)
    for label, skill_dir in todo:
        print(f"- {label}/{skill_dir.name}", flush=True)

    ok = 0
    err = 0
    started = time.time()
    with out_path.open(mode) as handle:
        for idx, (label, skill_dir) in enumerate(todo, 1):
            record: dict[str, Any] = {
                "n": idx,
                "label": label,
                "skill_dir": str(skill_dir),
                "skill_name": skill_dir.name,
                "_retry": False,
            }
            t0 = time.time()
            try:
                result = run_smoke(str(skill_dir))
                record.update(result)
                record["error"] = None
                ok += 1
            except Exception as exc:
                record["error"] = f"{type(exc).__name__}: {exc}"
                record["traceback"] = traceback.format_exc()[-1200:]
                record["_retry"] = True
                err += 1
            record["elapsed_s"] = round(time.time() - t0, 1)
            handle.write(json.dumps(record) + "\n")
            handle.flush()
            elapsed_total = round(time.time() - started)
            print(
                f"[{idx}/{len(todo)}] {label}/{skill_dir.name} -> "
                f"verdict={record.get('verdict', 'ERR')} status={record.get('finalization_status', '?')} "
                f"hits={record.get('receiver_hits', '?')} shim={len(record.get('shim_calls', []))} "
                f"({record['elapsed_s']}s, total {elapsed_total}s, ok={ok} err={err})",
                flush=True,
            )

    print(f"DONE. {ok} ok, {err} errors, {round(time.time() - started)}s total.", flush=True)
    print(f"Results written to: {out_path}", flush=True)
    # Compute simple metrics (accuracy, precision, recall, F1) for this sample
    try:
        with out_path.open() as f2:
            all_results = [json.loads(line) for line in f2]
        successful = [r for r in all_results if r.get('error') is None]
        tp = tn = fp = fn = 0
        for r in successful:
            label = r.get('label')
            verdict = r.get('verdict')
            gt_safe = (label == 'benign')
            pred_safe = verdict in ['read_observed', 'not_observed']
            if gt_safe and pred_safe:
                tn += 1
            elif gt_safe and not pred_safe:
                fp += 1
            elif not gt_safe and not pred_safe:
                tp += 1
            elif not gt_safe and pred_safe:
                fn += 1

        total = tp + tn + fp + fn
        accuracy = (tp + tn) / total if total > 0 else 0
        precision = tp / (tp + fp) if (tp + fp) > 0 else 0
        recall = tp / (tp + fn) if (tp + fn) > 0 else 0
        f1 = 2 * (precision * recall) / (precision + recall) if (precision + recall) > 0 else 0

        metrics = {
            'total': total,
            'successful': len(successful),
            'errors': len(all_results) - len(successful),
            'tp': tp, 'tn': tn, 'fp': fp, 'fn': fn,
            'accuracy': accuracy,
            'precision': precision,
            'recall': recall,
            'f1': f1,
        }
        print("\nSAMPLE METRICS:")
        print(f"  total: {metrics['total']}  successful: {metrics['successful']}  errors: {metrics['errors']}")
        print(f"  TP={tp} TN={tn} FP={fp} FN={fn}")
        print(f"  Accuracy={accuracy:.3f} Precision={precision:.3f} Recall={recall:.3f} F1={f1:.3f}")

        metrics_path = out_path.with_suffix('.metrics.json')
        with metrics_path.open('w') as mf:
            json.dump(metrics, mf, indent=2)
        print(f"Metrics written to: {metrics_path}")
    except Exception as e:
        print(f"Failed to compute metrics: {e}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
