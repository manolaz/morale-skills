#!/usr/bin/env python3
"""
MORALE Benchmark Runner with Gemini LLM Analysis
Automatically scans skill folders, runs audits, and analyzes results using Gemini AI.
"""

import os
import sys
import json
import subprocess
import time
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, asdict
import google.generativeai as genai

# ===== CONFIGURATION =====
API_KEY = "AIzaSyCUI_bNrZQlyO78vOPzqxty_giljF94Ies"
LIMIT_BENIGN = 6  # Limit for testing (increase to 50 for full run)
LIMIT_MALWARE = 6  # Limit for testing (increase to 50 for full run)

MORALE_BINARY = "./target/debug/morale"  # Path to compiled morale binary
DATA_DIR = "./data/skills"  # Path to skills directory
OUTPUT_FILE = "benchmark_results_report.json"

# Configure Gemini API
genai.configure(api_key=API_KEY)
MODEL = genai.GenerativeModel("gemini-2.5-flash")

# ===== DATA STRUCTURES =====
@dataclass
class ExecutionTrace:
    """Stores execution trace from a single audit run"""
    skill_path: str
    skill_name: str
    classification: str  # "benign" or "malicious"
    stdout: str
    stderr: str
    exit_code: int
    execution_time: float
    risks_detected: int

@dataclass
class AnalysisResult:
    """LLM analysis result for a single trace"""
    skill_name: str
    classification: str
    actual_classification: str
    confidence: float
    reasoning: str
    risks_identified: List[str]

@dataclass
class BenchmarkMetrics:
    """Security metrics calculated from results"""
    total_audits: int
    true_positives: int
    true_negatives: int
    false_positives: int
    false_negatives: int
    accuracy: float
    true_positive_rate: float  # TPR / Sensitivity
    false_positive_rate: float  # FPR
    precision: float
    f1_score: float

# ===== EXECUTION PHASE =====

def find_skill_folders(classification: str) -> List[Path]:
    """Find all skill folders by classification (benign or malicious)"""
    folder_path = Path(DATA_DIR) / classification
    
    if not folder_path.exists():
        print(f"⚠️  Folder not found: {folder_path}")
        return []
    
    skills = sorted([d for d in folder_path.iterdir() if d.is_dir()])
    print(f"✓ Found {len(skills)} {classification} skills in {folder_path}")
    return skills

def run_morale_audit(skill_path: Path, classification: str) -> Optional[ExecutionTrace]:
    """Run morale binary on a skill folder and capture execution trace"""
    try:
        start_time = time.time()
        
        # Run morale in sandbox
        result = subprocess.run(
            [MORALE_BINARY, str(skill_path), "--json"],
            capture_output=True,
            text=True,
            timeout=30  # 30 second timeout per audit
        )
        
        execution_time = time.time() - start_time
        
        # Count risks from output
        risks_detected = 0
        try:
            output_json = json.loads(result.stdout)
            if isinstance(output_json, dict) and "risks" in output_json:
                risks_detected = len(output_json["risks"])
        except json.JSONDecodeError:
            pass
        
        trace = ExecutionTrace(
            skill_path=str(skill_path),
            skill_name=skill_path.name,
            classification=classification,
            stdout=result.stdout[:1000],  # Limit to 1000 chars
            stderr=result.stderr[:1000],
            exit_code=result.returncode,
            execution_time=execution_time,
            risks_detected=risks_detected
        )
        
        return trace
        
    except subprocess.TimeoutExpired:
        print(f"✗ Timeout for {skill_path.name}")
        return None
    except Exception as e:
        print(f"✗ Error running audit on {skill_path.name}: {e}")
        return None

def collect_execution_traces() -> Tuple[List[ExecutionTrace], List[ExecutionTrace]]:
    """Collect execution traces for all skills"""
    benign_traces = []
    malicious_traces = []
    
    print("\n" + "="*60)
    print("PHASE 1: Collecting Execution Traces")
    print("="*60)
    
    # Process benign skills
    print(f"\n📁 Processing BENIGN skills (limit: {LIMIT_BENIGN})...")
    benign_folders = find_skill_folders("benign")[:LIMIT_BENIGN]
    for i, skill_path in enumerate(benign_folders, 1):
        print(f"  [{i}/{len(benign_folders)}] Auditing {skill_path.name}...", end=" ")
        trace = run_morale_audit(skill_path, "benign")
        if trace:
            benign_traces.append(trace)
            print(f"✓ ({trace.risks_detected} risks, {trace.execution_time:.2f}s)")
        else:
            print("✗")
        time.sleep(0.5)  # Rate limiting
    
    # Process malicious skills
    print(f"\n📁 Processing MALICIOUS skills (limit: {LIMIT_MALWARE})...")
    malicious_folders = find_skill_folders("malicious")[:LIMIT_MALWARE]
    for i, skill_path in enumerate(malicious_folders, 1):
        print(f"  [{i}/{len(malicious_folders)}] Auditing {skill_path.name}...", end=" ")
        trace = run_morale_audit(skill_path, "malicious")
        if trace:
            malicious_traces.append(trace)
            print(f"✓ ({trace.risks_detected} risks, {trace.execution_time:.2f}s)")
        else:
            print("✗")
        time.sleep(0.5)  # Rate limiting
    
    print(f"\n✓ Collected {len(benign_traces)} benign + {len(malicious_traces)} malicious traces")
    return benign_traces, malicious_traces

# ===== ANALYSIS PHASE =====

def analyze_trace_with_gemini(trace: ExecutionTrace) -> Optional[AnalysisResult]:
    """Send execution trace to Gemini for analysis"""
    try:
        prompt = f"""
You are a security analyst. Analyze this MORALE audit execution trace and determine if the skill is BENIGN or MALICIOUS.

**Skill Name:** {trace.skill_name}
**Execution Time:** {trace.execution_time:.2f}s
**Exit Code:** {trace.exit_code}
**Risks Detected:** {trace.risks_detected}

**Audit Output (stdout):**
{trace.stdout}

**Audit Errors (stderr):**
{trace.stderr}

Based on this execution trace, answer in JSON format:
{{
    "classification": "benign" or "malicious",
    "confidence": 0.0 to 1.0,
    "reasoning": "brief explanation",
    "risks_identified": ["risk1", "risk2", ...]
}}
"""
        
        response = MODEL.generate_content(prompt)
        
        # Parse JSON from response
        json_start = response.text.find("{")
        json_end = response.text.rfind("}") + 1
        json_str = response.text[json_start:json_end]
        analysis_data = json.loads(json_str)
        
        result = AnalysisResult(
            skill_name=trace.skill_name,
            classification=analysis_data.get("classification", "unknown"),
            actual_classification=trace.classification,
            confidence=float(analysis_data.get("confidence", 0.5)),
            reasoning=analysis_data.get("reasoning", ""),
            risks_identified=analysis_data.get("risks_identified", [])
        )
        
        return result
        
    except Exception as e:
        print(f"  ✗ Gemini analysis failed: {e}")
        return None

def analyze_all_traces(benign_traces: List[ExecutionTrace], 
                      malicious_traces: List[ExecutionTrace]) -> Tuple[List[AnalysisResult], List[AnalysisResult]]:
    """Analyze all traces using Gemini"""
    benign_analyses = []
    malicious_analyses = []
    
    print("\n" + "="*60)
    print("PHASE 2: Gemini LLM Analysis")
    print("="*60)
    
    # Analyze benign traces
    print(f"\n🤖 Analyzing BENIGN traces ({len(benign_traces)} total)...")
    for i, trace in enumerate(benign_traces, 1):
        print(f"  [{i}/{len(benign_traces)}] Analyzing {trace.skill_name}...", end=" ")
        result = analyze_trace_with_gemini(trace)
        if result:
            benign_analyses.append(result)
            status = "✓" if result.classification == "benign" else "✗"
            print(f"{status} ({result.classification}, confidence: {result.confidence:.2f})")
        else:
            print("✗")
        time.sleep(1)  # Rate limiting for API
    
    # Analyze malicious traces
    print(f"\n🤖 Analyzing MALICIOUS traces ({len(malicious_traces)} total)...")
    for i, trace in enumerate(malicious_traces, 1):
        print(f"  [{i}/{len(malicious_traces)}] Analyzing {trace.skill_name}...", end=" ")
        result = analyze_trace_with_gemini(trace)
        if result:
            malicious_analyses.append(result)
            status = "✓" if result.classification == "malicious" else "✗"
            print(f"{status} ({result.classification}, confidence: {result.confidence:.2f})")
        else:
            print("✗")
        time.sleep(1)  # Rate limiting for API
    
    return benign_analyses, malicious_analyses

# ===== METRICS CALCULATION =====

def calculate_metrics(benign_analyses: List[AnalysisResult], 
                     malicious_analyses: List[AnalysisResult]) -> BenchmarkMetrics:
    """Calculate security metrics (Accuracy, TPR, FPR, etc.)"""
    
    tp = sum(1 for r in malicious_analyses if r.classification == "malicious")
    tn = sum(1 for r in benign_analyses if r.classification == "benign")
    fp = sum(1 for r in benign_analyses if r.classification == "malicious")
    fn = sum(1 for r in malicious_analyses if r.classification == "benign")
    
    total = len(benign_analyses) + len(malicious_analyses)
    
    accuracy = (tp + tn) / total if total > 0 else 0
    tpr = tp / len(malicious_analyses) if len(malicious_analyses) > 0 else 0  # Sensitivity
    fpr = fp / len(benign_analyses) if len(benign_analyses) > 0 else 0
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0
    f1 = 2 * (precision * tpr) / (precision + tpr) if (precision + tpr) > 0 else 0
    
    return BenchmarkMetrics(
        total_audits=total,
        true_positives=tp,
        true_negatives=tn,
        false_positives=fp,
        false_negatives=fn,
        accuracy=accuracy,
        true_positive_rate=tpr,
        false_positive_rate=fpr,
        precision=precision,
        f1_score=f1
    )

# ===== REPORTING PHASE =====

def print_metrics_report(metrics: BenchmarkMetrics):
    """Print metrics to terminal"""
    print("\n" + "="*60)
    print("PHASE 3: Security Metrics Report")
    print("="*60)
    
    print(f"\n📊 Confusion Matrix:")
    print(f"  True Positives (TP):   {metrics.true_positives}")
    print(f"  True Negatives (TN):   {metrics.true_negatives}")
    print(f"  False Positives (FP):  {metrics.false_positives}")
    print(f"  False Negatives (FN):  {metrics.false_negatives}")
    print(f"  Total Audits:          {metrics.total_audits}")
    
    print(f"\n🎯 Performance Metrics:")
    print(f"  Accuracy:              {metrics.accuracy:.4f} ({metrics.accuracy*100:.2f}%)")
    print(f"  Sensitivity (TPR):     {metrics.true_positive_rate:.4f} ({metrics.true_positive_rate*100:.2f}%)")
    print(f"  False Positive Rate:   {metrics.false_positive_rate:.4f} ({metrics.false_positive_rate*100:.2f}%)")
    print(f"  Precision:             {metrics.precision:.4f} ({metrics.precision*100:.2f}%)")
    print(f"  F1 Score:              {metrics.f1_score:.4f}")
    
    # Interpretation
    print(f"\n💡 Interpretation:")
    if metrics.accuracy > 0.9:
        print(f"  ✓ Excellent detection performance (Accuracy > 90%)")
    elif metrics.accuracy > 0.8:
        print(f"  ✓ Good detection performance (Accuracy > 80%)")
    else:
        print(f"  ⚠️  Detection performance needs improvement")
    
    if metrics.false_positive_rate < 0.1:
        print(f"  ✓ Low false positive rate (< 10%)")
    else:
        print(f"  ⚠️  High false positive rate - may need tuning")

def save_json_report(benign_analyses: List[AnalysisResult],
                    malicious_analyses: List[AnalysisResult],
                    metrics: BenchmarkMetrics):
    """Save detailed report to JSON file"""
    
    report = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "configuration": {
            "benign_limit": LIMIT_BENIGN,
            "malware_limit": LIMIT_MALWARE,
            "api_model": "gemini-2.5-flash"
        },
        "benign_analyses": [asdict(a) for a in benign_analyses],
        "malicious_analyses": [asdict(a) for a in malicious_analyses],
        "metrics": asdict(metrics)
    }
    
    with open(OUTPUT_FILE, "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"\n💾 Detailed report saved to {OUTPUT_FILE}")

# ===== MAIN EXECUTION =====

def main():
    print("""
╔════════════════════════════════════════════════════════════╗
║        MORALE AI Skills Auditor - Benchmark Runner         ║
║              Powered by Gemini LLM Analysis                ║
╚════════════════════════════════════════════════════════════╝
""")
    
    # Check if morale binary exists
    if not os.path.exists(MORALE_BINARY):
        print(f"❌ Error: MORALE binary not found at {MORALE_BINARY}")
        print("   Please build the project first: cargo build")
        sys.exit(1)
    
    # Check if data directory exists
    if not os.path.exists(DATA_DIR):
        print(f"❌ Error: Data directory not found at {DATA_DIR}")
        sys.exit(1)
    
    try:
        # Phase 1: Collect execution traces
        benign_traces, malicious_traces = collect_execution_traces()
        
        if not benign_traces and not malicious_traces:
            print("❌ No execution traces collected. Exiting.")
            sys.exit(1)
        
        # Phase 2: Analyze with Gemini
        benign_analyses, malicious_analyses = analyze_all_traces(benign_traces, malicious_traces)
        
        if not benign_analyses and not malicious_analyses:
            print("❌ No analyses completed. Exiting.")
            sys.exit(1)
        
        # Phase 3: Calculate metrics
        metrics = calculate_metrics(benign_analyses, malicious_analyses)
        print_metrics_report(metrics)
        
        # Phase 4: Save report
        save_json_report(benign_analyses, malicious_analyses, metrics)
        
        print("\n" + "="*60)
        print("✓ Benchmark run completed successfully!")
        print("="*60)
        
    except KeyboardInterrupt:
        print("\n\n⚠️  Benchmark interrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n❌ Error during benchmark: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()
