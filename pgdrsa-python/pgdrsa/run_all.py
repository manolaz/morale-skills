"""Run pgdrsa.smoke over all 100 skills and write JSONL results.

Usage:
    python -m pgdrsa.run_all [--limit N] [--retry-errors] [--results-file FILE] [--image IMG] [--only-bad]

Resume: if the results file exists, skills already in it are skipped.

To re-run only the failures from a baseline:
    python -m pgdrsa.run_all --results-file pgdrsa_results_enriched.jsonl \\
        --only-bad pgdrsa_results.jsonl --image pgdrsa_sandbox:phaseA-enriched
"""
from __future__ import annotations
import argparse, json, os, sys, time, traceback
from pathlib import Path
from . import config as _config
from .smoke import run_smoke

SKILLS_DIR = Path("data/skills")


def _enumerate() -> list[tuple[str, Path]]:
    out: list[tuple[str, Path]] = []
    for cls in ["benign", "malicious"]:
        for d in sorted((SKILLS_DIR / cls).iterdir()):
            if d.is_dir() and (d / "SKILL.md").exists():
                out.append((cls, d))
    return out


def _already_done(out_path: Path) -> set[str]:
    if not out_path.exists():
        return set()
    done: set[str] = set()
    for line in out_path.read_text().splitlines():
        try:
            rec = json.loads(line)
            if rec.get("error") is None or not rec.get("_retry"):
                done.add(rec["skill_dir"])
        except json.JSONDecodeError:
            pass
    return done


def _bad_skills(baseline_path: Path) -> set[str]:
    """Skills that errored OR were false-negative predictions in the baseline.

    False-negative = label was 'malicious' but the verdict was not executed_contained
    (i.e., the skill failed to land a canary — that's what the enrichment is meant to fix).
    """
    if not baseline_path.exists():
        return set()
    bad: set[str] = set()
    for line in baseline_path.read_text().splitlines():
        try:
            rec = json.loads(line)
        except json.JSONDecodeError:
            continue
        if rec.get("error"):
            bad.add(rec["skill_dir"])
        elif rec.get("label") == "malicious" and rec.get("verdict") != "executed_contained":
            bad.add(rec["skill_dir"])
    return bad


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0, help="max skills to run (0=all)")
    ap.add_argument("--retry-errors", action="store_true", help="re-run skills that errored")
    ap.add_argument("--results-file", default="pgdrsa_results.jsonl",
                    help="JSONL output path (default: pgdrsa_results.jsonl)")
    ap.add_argument("--image", help="override PGDRSA_IMAGE_REF for this run")
    ap.add_argument("--only-bad", metavar="BASELINE_JSONL",
                    help="only run skills that errored or were FN in the given baseline")
    args = ap.parse_args()

    if args.image:
        os.environ["PGDRSA_IMAGE_REF"] = args.image
        _config.IMAGE_REF = args.image
        # re-import smoke to pick up the new value
        from importlib import reload
        from . import smoke
        reload(smoke)
        globals()["run_smoke"] = smoke.run_smoke
        print(f"Using image: {args.image}", flush=True)

    out_path = Path(args.results_file)
    skills = _enumerate()
    done = _already_done(out_path) if not args.retry_errors else set()
    if args.only_bad:
        bad = _bad_skills(Path(args.only_bad))
        todo = [(c, d) for c, d in skills if str(d) in bad and str(d) not in done]
        print(f"--only-bad: {len(bad)} failed/FN skills in {args.only_bad}; "
              f"{len(todo)} to re-run after resume.", flush=True)
    else:
        todo = [(c, d) for c, d in skills if str(d) not in done]
    if args.limit:
        todo = todo[: args.limit]
    print(f"Total skills: {len(skills)} | already done in {out_path.name}: {len(done)} | "
          f"todo: {len(todo)}", flush=True)

    mode = "a" if (done or out_path.exists()) else "w"
    successes = errors = 0
    t_start = time.time()
    with out_path.open(mode) as f:
        for i, (cls, d) in enumerate(todo, 1):
            rec: dict = {"n": len(done) + i, "label": cls, "skill_dir": str(d),
                         "skill_name": d.name, "_retry": False}
            t0 = time.time()
            try:
                result = run_smoke(str(d))
                rec.update(result)
                rec["error"] = None
                successes += 1
            except Exception as e:
                rec["error"] = f"{type(e).__name__}: {e}"
                rec["traceback"] = traceback.format_exc()[-800:]
                rec["_retry"] = True
                errors += 1
            rec["elapsed_s"] = round(time.time() - t0, 1)
            f.write(json.dumps(rec) + "\n"); f.flush()
            elapsed_total = round(time.time() - t_start)
            shim_n = len(rec.get("shim_calls", []))
            print(f"[{len(done)+i}/{len(skills)}] {cls}/{d.name} -> "
                  f"verdict={rec.get('verdict','ERR')} status={rec.get('finalization_status','?')} "
                  f"hits={rec.get('receiver_hits','?')} shim={shim_n} ({rec['elapsed_s']}s, "
                  f"total {elapsed_total}s, ok={successes} err={errors})", flush=True)
    print(f"DONE. {successes} ok, {errors} errors, {round(time.time()-t_start)}s total.", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
