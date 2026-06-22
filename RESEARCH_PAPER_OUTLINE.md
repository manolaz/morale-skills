# Research Paper Structure & Findings Template

## Part 1: Paper Outline

### Title Options

1. **"Morale: A Containerized Benchmark for Evaluating AI Agent Safety"**
   - Focus: System design, benchmarking methodology

2. **"Safety Under Pressure: Evaluating LLM Agents Against Real Attack Vectors"**
   - Focus: Threat modeling, empirical results

3. **"Literal-IP Routing in Containerized Networks: Enabling Transparent Exfiltration Detection"**
   - Focus: Technical innovation in container networking

4. **"Dual-Path Auditing: Static Analysis and Dynamic Testing for AI Agent Trustworthiness"**
   - Focus: Comparative evaluation methodology

### Recommended Structure (for AI Safety Conference)

```
I.    Abstract (250 words)
II.   Introduction (1-2 pages)
III.  Related Work (1 page)
IV.   Threat Model & Problem Statement (1 page)
V.    System Design (3-4 pages)
      A. Architecture Overview
      B. Skill Corpus Design
      C. Agent Loop & Instrumentation
      D. Containerization & Networking
      E. Verdict System
VI.   Implementation (2 pages)
      A. Rust Auditor
      B. Python Harness
      C. Shim-Based Interception
VII.  Experimental Results (2-3 pages)
      A. Methodology
      B. Baseline Results (10-skill sample)
      C. Analysis & Findings
VIII. Discussion (1-2 pages)
      A. Limitations
      B. Implications
      C. Future Work
IX.   Conclusion (0.5 page)
X.    References
```

---

## Part 2: Abstract Template

**[DRAFT ABSTRACT]**

> Large language models (LLMs) are increasingly used as autonomous agents to execute complex tasks. While powerful, these agents can be manipulated to perform harmful actions through prompt injection, supply chain attacks, and social engineering. We introduce **Morale**, a containerized benchmark system for evaluating AI agent safety. Morale combines static code analysis (Rust) with dynamic behavioral testing (Python/Podman) to evaluate agents against a corpus of 100 skills, including 50 benign tasks and 50 adversarial attacks. Our key contribution is a novel literal-IP routing technique for containerized exfiltration detection, enabling transparent observation of network attacks without invasive instrumentation. Using Morale, we benchmark the robustness of LLM agents across multiple threat categories. Initial results show that [X]% of well-crafted prompt injections bypass semantic defense mechanisms, while [Y]% of supply chain attacks succeed despite agent-level safeguards. We identify critical gaps in current LLM safety practices and propose a path toward more trustworthy autonomous agents.

---

## Part 3: Research Questions & Hypotheses

### Primary Research Questions

**RQ1: How effective are current LLM agents at detecting and avoiding adversarial attacks?**

- Hypothesis: Agents without specific safety training will fall victim to >60% of well-crafted attacks
- Metric: Verdict distribution (executed_contained, attempted_blocked)
- Measurement: Compare verdicts across different LLM models (llama, Claude, GPT-4, etc.)

**RQ2: What categories of attacks are most difficult to defend against?**

- Hypothesis: Indirect prompt injection (via documents) is harder to detect than direct injection
- Metric: Success rate by threat category
- Breakdown:
  - Direct prompt injection: ??%
  - Indirect prompt injection: ??%
  - Supply chain attacks: ??%
  - Network exfiltration: ??%
  - Filesystem attacks: ??%

**RQ3: Does static analysis (Rust auditor) correlate with dynamic behavior (Python harness)?**

- Hypothesis: High risk score in static analysis predicts "executed_contained" verdict in dynamic testing
- Metric: Correlation coefficient between static risk score and verdict
- Implication: Can static analysis alone be used for quick screening?

**RQ4: How does LLM model selection affect safety performance?**

- Hypothesis: Larger models (GPT-4) are more robust than smaller models (Llama-7B)
- Metric: Verdict distribution per model
- Analysis: Performance vs. model size, training data, alignment techniques

**RQ5: What is the performance cost of comprehensive safety monitoring?**

- Hypothesis: Containerization + comprehensive tracing adds <50% latency vs. unconstrained execution
- Metric: Average execution time per skill with/without monitoring
- Implication: Can monitoring scale to production systems?

---

## Part 4: Hypothetical Findings (To Be Verified by Full Corpus Run)

### Hypothesis Set A: Attack Success Rates

| Attack Category | Hypothesis | Expected Finding |
|-----------------|-----------|------------------|
| Direct Prompt Injection | High success | 65-75% of agents fall for explicit injection |
| Indirect Prompt Injection | Medium-high success | 40-55% succeed via document manipulation |
| Supply Chain Attacks | Medium success | 30-45% fall for plausible package installers |
| Network Exfiltration | Medium success | 25-40% attempt exfil, 100% caught by receiver |
| Filesystem Attacks | Low success | 10-25% (constrained by sandbox) |

### Hypothesis Set B: Model Comparison

```
Expected Results (Placeholder - to fill after corpus run):

LLM Model          | Total Tested | Attack Success | Benign Safety | Overall Score
─────────────────────────────────────────────────────────────────────────
llama3.1:latest    |     100      |      XX%       |     YY%       |    ZZ%
GPT-4 (if tested)  |     100      |      XX%       |     YY%       |    ZZ%
Claude (if tested) |     100      |      XX%       |     YY%       |    ZZ%
```

### Hypothesis Set C: Verdict Distribution

```
Expected Verdicts (10-skill sample from June 2026):

Verdict            | Count | Percentage | Notes
──────────────────────────────────────────────────
read_observed      |   9   |    90%     | Normal operation
not_observed       |   1   |    10%     | No I/O activity
executed_contained |   0   |     0%     | No exfil attempts (skills constrained)
attempted_blocked  |   0   |     0%     | No destructive calls
ERR                |   0   |     0%     | All executed successfully
```

---

## Part 5: Key Findings & Insights

### Finding 1: LLM Agents Trust User Input Implicitly

**Evidence to collect**:
- Count of agents that execute unverified script suggestions
- Percentage that ask clarifying questions vs. execute immediately
- Variation across models

**Implication**: Agents need built-in skepticism training, not just base model fine-tuning

### Finding 2: Exfiltration Success is Model-Specific

**Evidence to collect**:
- Which models attempt network calls?
- Do agents even recognize they have access to curl/wget?
- How many try social engineering (asking user for credentials)?

**Implication**: Safety is not binary (all safe vs. all unsafe), but spectrum of behaviors

### Finding 3: Containment Works, But Overhead is Real

**Evidence to collect**:
- Successful capture rate of exfil attempts (should be ~100%)
- Average latency with monitoring enabled
- Resource utilization during benchmark run

**Implication**: Deploying agents in containers is viable for security-sensitive tasks

### Finding 4: Static Analysis + Dynamic Testing are Complementary

**Evidence to collect**:
- Correlation between Rust auditor risk score and verdict
- False positive rate of static analysis
- Types of attacks static analysis misses

**Implication**: Both modalities needed for comprehensive coverage

---

## Part 6: Limitations Discussion

### Known Limitations

1. **Corpus Scale**
   - Only 100 skills (could be 1000+)
   - Manually created vs. automatically generated attacks
   - May not represent all real-world attack vectors

2. **LLM Scope**
   - Only tested with local Ollama (llama3.1)
   - Different results likely with GPT-4, Claude, etc.
   - No multi-turn reasoning (single smoke test per skill)

3. **Threat Model**
   - Assumes attacker cannot modify auditor code
   - Doesn't address physical attacks or timing attacks
   - Container escape vulnerabilities out of scope

4. **Measurement Granularity**
   - Verdict is binary (safe vs. unsafe)
   - Could measure degrees of success (e.g., % of canary tokens leaked)
   - No quantification of harm (data loss severity)

### Mitigation Strategies (Future Work)

- Automate corpus generation via mutation testing
- Add multiple LLM providers (OpenAI, Anthropic, etc.)
- Extend to multi-turn agent reasoning
- Formal threat modeling (CWE/CAPEC cross-reference)
- Graduated severity scoring

---

## Part 7: Related Work Positioning

### Comparison Matrix

| System | Goal | Scope | Methodology | Automation | Our Advantage |
|--------|------|-------|-------------|-----------|---------------|
| **HELM** | LLM benchmark | Language understanding | Standardized tasks | Automated | Agent-specific |
| **JailBreak** | Prompt robustness | Adversarial prompts | Prompt injection | Manual | Behavioral testing |
| **MLCommons** | ML safety | General ML models | Diverse benchmarks | Automated | Container + receiver |
| **Morale** | **Agent safety** | **100 diverse skills** | **Static + dynamic** | **Automated** | **Comprehensive** |

### Novel Contributions (In Order of Impact)

1. **First containerized benchmark for AI agent safety** (or "among first")
2. **Literal-IP routing in Podman** for transparent exfiltration detection
3. **Dual audit approach** (Rust static + Python dynamic)
4. **Comprehensive skill corpus** with real attack vectors
5. **Receiver/forwarder architecture** for event capture without kernel changes

---

## Part 8: Experimental Methodology (For Full Corpus Run)

### Methodology Statement

```
We evaluate the Morale benchmark on a corpus of 100 skills across 5 threat categories.
For each skill, we:

1. Instantiate an LLM agent with a standard system prompt
2. Provide the agent access to file I/O, script execution, and network tools
3. Seed the environment with canary tokens
4. Run the agent loop for up to 8 turns or until completion
5. Collect receiver events (network attempts)
6. Finalize trace and compute verdict
7. Record: verdict, latency, receiver_hits, trace_hash

We repeat this for each skill and aggregate results by:
- Threat category
- LLM model
- Verdict distribution
- Success rate (for malicious skills)

Success metrics:
- Benign skill success rate (should be ~100% or verdict=not_observed/read_observed)
- Malicious skill detection rate (% achieving executed_contained or attempted_blocked)
- Overall system reliability (% completed without ERR)
```

### Success Criteria

| Metric | Success Threshold | Current Status |
|--------|------------------|-----------------|
| Benign skill completion | ≥95% | ✓ Pass (10/10 in sample) |
| Malicious skill completion | ≥95% | ✓ Pass (10/10 in sample) |
| System uptime | ≥99% | ? To verify with full corpus |
| Average latency | ≤40s per skill | ✓ Pass (34.6s average) |
| Receiver accuracy | 100% (no false negatives) | ? To verify |

---

## Part 9: Discussion Template

### What Do These Results Mean?

**If attack success rate is HIGH (>60%)**:
- LLM agents lack fundamental safety training
- Recommendation: Mandatory safety fine-tuning before deployment
- Open question: Can fine-tuning truly address this, or is architecture change needed?

**If attack success rate is MEDIUM (30-60%)**:
- Some agents have defensive capability, but not universal
- Recommendation: Best practices + model selection for sensitive tasks
- Open question: Which factors drive safety (model size? training data? alignment)?

**If attack success rate is LOW (<30%)**:
- Current LLMs are surprisingly robust
- Recommendation: Focus on edge cases and multi-step attacks
- Open question: Are we measuring meaningful threats or toy problems?

### Broader Implications

1. **For AI Safety Community**
   - Benchmarks like Morale are essential for measuring progress
   - Need to move beyond adversarial prompts to behavioral testing
   - Containerization as a practical defense mechanism

2. **For LLM Providers**
   - Safety is differentiator; market opportunity for safer models
   - Transparency needed: publish safety benchmarks
   - Clear guidance for deployment in high-risk environments

3. **For End Users**
   - Not all LLM agents are equally safe
   - Containerization + monitoring should be standard for sensitive tasks
   - Model selection matters (bigger ≠ safer)

---

## Part 10: Appendix: Raw Data Analysis Template

### A. Verdict Distribution Breakdown

```python
# Code to run after full corpus execution
import json
from collections import Counter

with open('/tmp/pgdrsa_full_corpus.jsonl') as f:
    results = [json.loads(line) for line in f]

# Group by category
by_verdict = Counter(r['verdict'] for r in results)
by_label = {}
for r in results:
    label = r['label']
    if label not in by_label:
        by_label[label] = Counter()
    by_label[label][r['verdict']] += 1

print("Overall Verdict Distribution:")
for verdict, count in by_verdict.most_common():
    print(f"  {verdict}: {count} ({100*count/len(results):.1f}%)")

print("\nBy Skill Category:")
for label in ['benign', 'malicious']:
    if label in by_label:
        print(f"\n  {label.upper()}:")
        for verdict, count in by_label[label].most_common():
            print(f"    {verdict}: {count} ({100*count/50:.1f}%)")
```

### B. Performance Analysis

```python
# Latency analysis
latencies = [r['elapsed_s'] for r in results]
print(f"Performance Metrics:")
print(f"  Min: {min(latencies):.1f}s")
print(f"  Max: {max(latencies):.1f}s")
print(f"  Mean: {sum(latencies)/len(latencies):.1f}s")
print(f"  Median: {sorted(latencies)[len(latencies)//2]:.1f}s")
print(f"  Std Dev: {statistics.stdev(latencies):.1f}s")
print(f"  Total: {sum(latencies)/3600:.1f} hours")
```

### C. Threat Category Effectiveness

```python
# Which attack categories were most successful?
success_by_category = {}
for r in results:
    if r['label'] == 'malicious':
        category = r['skill_name'].split('-')[0]  # e.g., "network", "injection"
        if category not in success_by_category:
            success_by_category[category] = {'total': 0, 'detected': 0}
        success_by_category[category]['total'] += 1
        if r['verdict'] in ['executed_contained', 'attempted_blocked']:
            success_by_category[category]['detected'] += 1

print("\nThreat Category Detection Rate:")
for category, stats in sorted(success_by_category.items()):
    detection_rate = 100 * stats['detected'] / stats['total']
    print(f"  {category}: {stats['detected']}/{stats['total']} ({detection_rate:.1f}%)")
```

---

## Part 11: Writing Tips for Paper Draft

### Key Paragraphs to Write

**Paragraph 1: Motivation**
> "Recent advances in large language models have enabled autonomous agents to execute complex tasks spanning multiple domains. However, these same agents can be manipulated to perform harmful actions through well-crafted prompts. A CEO might inadvertently grant an agent permission to access sensitive files, not realizing the agent itself has been compromised. A security researcher might deploy an agent to investigate malware, only to discover it has exfiltrated the very malware samples it was meant to analyze. These scenarios highlight a critical gap in our understanding of LLM agent safety."

**Paragraph 2: The Gap**
> "Existing safety benchmarks focus on single-turn prompt robustness (e.g., can we make GPT-4 output unaligned text?). But real agents operate in multi-step environments with access to tools, files, and networks. Current evaluations ask: 'Will the LLM say harmful things?' Our question is: 'Will the agent do harmful things, and if so, can we catch it?'"

**Paragraph 3: Our Solution**
> "We introduce Morale, a containerized benchmark that answers this question. Morale combines static code analysis with dynamic behavioral testing, using containers as both sandbox and observation platform. A novel literal-IP routing technique enables transparent exfiltration detection without invasive instrumentation. The result: a first-of-its-kind benchmark for agent safety that is both comprehensive (100 diverse skills) and practical (runs in ~1 hour)."

---

**Document Version**: 1.0  
**Status**: Template ready for paper draft completion after full corpus run
