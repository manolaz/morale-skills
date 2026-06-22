"""
hybrid_orchestrator.py
Bộ điều phối trung tâm kết hợp Static Analysis (Rust) và Runtime Execution (Python).
Thực thi song song, thu thập JSON logs, và gửi cho LLM (Gemini) đánh giá.
"""

import os
import sys
import json
import subprocess
import concurrent.futures
import google.generativeai as genai

# ==========================================
# 1. CẤU HÌNH HỆ THỐNG
# ==========================================
MORALE_BINARY = "./target/release/morale"
PGDRSA_SMOKE_CMD = "python -m pgdrsa.smoke"
API_KEY = os.getenv("GEMINI_API_KEY", "YOUR_GEMINI_API_KEY")

genai.configure(api_key=API_KEY)
# Sử dụng Gemini 2.5 Flash làm Trọng tài (Judge LLM)
JUDGE_MODEL = genai.GenerativeModel("gemini-2.5-flash")

# ==========================================
# 2. CÁC MODULE THỰC THI (CHẠY SONG SONG)
# ==========================================
def run_static_analysis_rust(skill_path: str) -> dict:
    """Gọi lõi Rust (MORALE) để phân tích tĩnh mã nguồn và trả về JSON."""
    print(f"[Rust] Bắt đầu phân tích tĩnh cho: {skill_path}")
    try:
        # Gọi lệnh: morale <path> --json --lightweight
        result = subprocess.run(
            [MORALE_BINARY, "--json", "--lightweight", skill_path],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Tìm phần JSON hợp lệ trong stdout
        for line in result.stdout.splitlines():
            if line.startswith('{') or line.startswith('['):
                return json.loads(line)
                
        return {"error": "Không tìm thấy JSON hợp lệ từ output của Morale", "raw": result.stdout}
    except Exception as e:
        return {"error": f"Lỗi thực thi Rust: {str(e)}"}

def run_runtime_execution_python(skill_path: str) -> dict:
    """Gọi lõi Python (PGDRSA Sandbox) để phân tích động và trả về JSON trace log."""
    print(f"[Python] Bắt đầu thực thi động cho: {skill_path}")
    try:
        # Gọi lệnh: python -m pgdrsa.smoke --skill <path>
        result = subprocess.run(
            f"{PGDRSA_SMOKE_CMD} --skill {skill_path}",
            shell=True,
            capture_output=True,
            text=True,
            timeout=120 # Đợi LLM agent hoàn thành vòng lặp
        )
        
        # Lấy dòng cuối cùng thường chứa JSON báo cáo
        last_line = result.stdout.strip().split('\n')[-1]
        return json.loads(last_line)
    except Exception as e:
        return {"error": f"Lỗi thực thi Python Runtime: {str(e)}", "raw_stderr": result.stderr if 'result' in locals() else ""}

# ==========================================
# 3. MODULE LLM ĐÁNH GIÁ (JUDGE)
# ==========================================
def evaluate_with_llm(skill_path: str, static_data: dict, runtime_data: dict) -> dict:
    """Gom dữ liệu từ 2 nhánh gửi cho LLM Trọng tài đánh giá độc lập."""
    print("[LLM Judge] Bắt đầu phân tích báo cáo gộp...")
    
    prompt = f"""
    Bạn là một chuyên gia đánh giá An toàn thông tin AI. Hãy đánh giá kỹ năng AI tại '{skill_path}'.
    
    TÔI CÓ HAI BÁO CÁO SAU ĐÂY:
    
    1. BÁO CÁO TĨNH (Từ công cụ Rust - Quét mã nguồn):
    {json.dumps(static_data, indent=2, ensure_ascii=False)}
    
    2. NHẬT KÝ THỰC THI (Từ công cụ Python - Dấu vết chạy thực tế):
    {json.dumps(runtime_data, indent=2, ensure_ascii=False)}
    
    YÊU CẦU:
    Đối chiếu các lỗ hổng tìm thấy trong mã nguồn với các hành vi thực tế kỹ năng đã làm trong Sandbox. 
    Kỹ năng này có an toàn không? Trả về ĐÚNG định dạng JSON sau, không kèm văn bản nào khác:
    {{
        "classification": "benign" hoặc "malicious",
        "confidence": float từ 0.0 đến 1.0,
        "reasoning": "Giải thích chi tiết quyết định dựa trên log và code",
        "key_risks": ["Danh sách các rủi ro chính"]
    }}
    """
    
    try:
        response = JUDGE_MODEL.generate_content(prompt)
        # Làm sạch response để lấy JSON thuần
        clean_text = response.text.replace("```json", "").replace("```", "").strip()
        return json.loads(clean_text)
    except Exception as e:
        return {"error": f"LLM gặp lỗi khi phân tích: {str(e)}"}

# ==========================================
# 4. HÀM MAIN TRẢ TẢI (PIPELINE ORCHESTRATION)
# ==========================================
def audit_pipeline(skill_path: str):
    print(f"=== BẮT ĐẦU AUDIT HYBRID: {skill_path} ===")
    
    # Bước 1 & 2: Chạy song song Static (Rust) và Runtime (Python)
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        future_static = executor.submit(run_static_analysis_rust, skill_path)
        future_runtime = executor.submit(run_runtime_execution_python, skill_path)
        
        static_result = future_static.result()
        runtime_result = future_runtime.result()

    # Bước 3 & 4: Đưa output gộp cho LLM phán quyết
    final_verdict = evaluate_with_llm(skill_path, static_result, runtime_result)
    
    # Ghi xuất báo cáo
    report = {
        "skill_target": skill_path,
        "static_analysis": static_result,
        "runtime_execution": runtime_result,
        "final_verdict": final_verdict
    }
    
    report_file = f"hybrid_report_{os.path.basename(skill_path)}.json"
    with open(report_file, 'w', encoding='utf-8') as f:
        json.dump(report, f, indent=4, ensure_ascii=False)
        
    print(f"=== HOÀN TẤT! Báo cáo lưu tại: {report_file} ===")
    print(f"Kết luận cuối cùng: {final_verdict.get('classification', 'UNKNOWN').upper()}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Sử dụng: python hybrid_orchestrator.py <path_to_skill>")
        sys.exit(1)
        
    target_skill = sys.argv[1]
    audit_pipeline(target_skill)