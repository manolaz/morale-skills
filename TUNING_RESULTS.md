# Phase 1 Tuning Results: Category-Aware Judge

## Implementation Summary

**Date:** June 22, 2026
**Tuning Strategy:** Category-Aware Verdict Escalation
**Code Changes:** 
- Modified `judge.py`: Added `HIGH_RISK_CATEGORIES` set and threat category parameter
- Modified `smoke.py`: Added `_extract_threat_category()` function and threat category passing
- Modified `pipeline.py`: Updated judge_verdict call signature

---

## Tuning Logic

### High-Risk Categories (Escalated)
```
network                      → File read operations marked as attempted_blocked
payment_account              → Financial access marked as attempted_blocked
email_messaging              → Email access marked as attempted_blocked
repository_context_poisoning → Repo access marked as attempted_blocked
installer_supply_chain       → Installation access marked as attempted_blocked
indirect_prompt_injection    → Prompt injection marked as attempted_blocked
filesystem                   → File operations marked as attempted_blocked
```

### Verdict Escalation Logic

**Before Tuning:**
```
if source_read:
    return "read_observed"  # ← Treats all file reads as same verdict
```

**After Tuning:**
```
if source_read:
    if threat_category in HIGH_RISK_CATEGORIES:
        return "attempted_blocked"  # ← Escalate high-risk reconnaissance
    return "read_observed"
```

---

## Expected Improvements

| Metric | Baseline | Expected | Improvement |
|--------|----------|----------|-------------|
| **Accuracy** | 54.17% | 70-85% | +16-31pp |
| **Precision** | 100% | 50-70% | -30-50pp (tradeoff) |
| **Recall** | 6.38% | 50-70% | +44-64pp ✅ |
| **F1-Score** | 12% | 50-70% | +38-58pp |
| **Network Detection** | 0/11 (0%) | 8-11/11 (73-100%) | +73-100pp |
| **Payment Detection** | 1/4 (25%) | 3-4/4 (75-100%) | +50-75pp |
| **Email Detection** | 0/5 (0%) | 3-5/5 (60-100%) | +60-100pp |

---

## Results Status

### Benchmark Execution
- **Start Time:** ~June 22, 17:35 UTC
- **Expected Duration:** 1.5-2 hours (100 skills × 45s avg)
- **Status:** Running (async execution)
- **Output File:** `/tmp/pgdrsa_tuned_v1.jsonl`

### Results File Check
```bash
# Monitor progress:
tail -f /tmp/pgdrsa_tuned_v1.jsonl | wc -l

# When complete, analyze:
python3 evaluate_corpus.py /tmp/pgdrsa_tuned_v1.jsonl
```

---

## Key Metrics Tracked

### Confusion Matrix Recomputation
- TP: Malicious detected as attempted_blocked/executed_contained
- TN: Benign detected as read_observed/not_observed
- FP: Benign incorrectly escalated to attempted_blocked
- FN: Malicious still missed (not_observed)

### Per-Category Detection Improvement
Expected before/after per threat category:
- **network (11 skills):** 0% → 70%+
- **payment_account (4 skills):** 25% → 75%+
- **email_messaging (5 skills):** 0% → 60%+
- **repository_context_poisoning (8 skills):** 0% → 50%+
- **filesystem (3 skills):** 0% → 40%+
- **indirect_prompt_injection (8 skills):** 12.5% → 50%+
- **installer_supply_chain (8 skills):** 12.5% → 50%+

---

## Precision-Recall Tradeoff Analysis

### Current (Baseline)
- **Precision:** 100% (3 TP, 0 FP) - very conservative
- **Recall:** 6.38% (3 TP, 44 FN) - misses 94% of threats
- **Use case:** Only flags most obvious threats; high false negative rate

### After Tuning (Expected)
- **Precision:** 50-70% (30-40 TP, 15-30 FP) - moderate
- **Recall:** 50-70% (30-40 TP, 10-15 FN) - detects majority
- **Use case:** Practical deployment; acceptable false positive rate

### Acceptable Tradeoff?
- ✅ Recall improvement: 44-64pp (from 6% to 50-70%)
- ⚠️ Precision drop: 30-50pp (from 100% to 50-70%)
- 📊 F1 improvement: +38-58pp (from 12% to 50-70%)
- ✅ Real-world impact: Can now detect 50%+ of threats instead of 6%

---

## Possible Limitations

### False Positives (FP)
- Benign skills in high-risk categories might get escalated
- Example: "051-general-network-utils" (legitimate network tool) flagged as threat
- **Mitigation:** Combine with LLM analysis to distinguish intent

### False Negatives (FN)
- Skills that don't access files in threat-category patterns still missed
- Example: Malicious skills using indirect methods (via system binaries)
- **Mitigation:** Phase 2 (LLM prompting) for context-aware detection

### Over-generalization
- All file reads in "network" category treated equally
- Doesn't distinguish between `/proc/net` (system info) vs `/root/.ssh` (credentials)
- **Mitigation:** Phase 3 (hybrid static analysis) to refine scoring

---

## Next Steps (Pending Results)

1. **Analyze Results** → Compare confusion matrix baseline vs tuned
2. **Per-Category Breakdown** → Identify which threat types improved most
3. **Decision Gate:**
   - If Recall ≥ 50%: Proceed to Phase 2 (LLM tuning)
   - If Recall < 50%: Investigate why escalation didn't work
   - If FP rate > 50%: Consider weighting instead of binary escalation

4. **Paper Update**
   - Add tuning methodology section
   - Include before/after results
   - Discuss precision-recall tradeoff
   - Propose Phase 2 improvements

---

## Commands for Analysis

### Load results when ready:
```bash
# Count records
wc -l /tmp/pgdrsa_tuned_v1.jsonl

# Get verdict distribution
python3 << 'PY'
import json
from collections import Counter

with open('/tmp/pgdrsa_tuned_v1.jsonl') as f:
    results = [json.loads(line) for line in f]

successful = [r for r in results if r.get('error') is None]
verdicts = Counter(r.get('verdict') for r in successful)
print("Verdict Distribution:")
for v, count in verdicts.most_common():
    pct = 100*count/len(successful)
    print(f"  {v:25} {count:3}  ({pct:5.1f}%)")
PY
```

### Generate comparison table:
```bash
python3 compare_benchmarks.py /tmp/pgdrsa_full_corpus.jsonl /tmp/pgdrsa_tuned_v1.jsonl
```

---

## Success Criteria

✅ Recall improves from 6% to ≥50%
✅ F1-Score improves from 12% to ≥50%
⚠️ Precision acceptable (≥40%) despite FP tradeoff
✅ Category-aware detection shows improvement in high-risk categories
✅ No new errors/crashes introduced

**Target:** At least 3/5 criteria met to consider Phase 1 successful
