# Morale Hybrid Auditor

Repo này chứa bộ công cụ đánh giá an toàn cho AI agent theo hai lớp:

- `morale-rust/`: auditor tĩnh bằng Rust, quét skill, chấm rủi ro và sinh báo cáo.
- `pgdrsa-python/`: harness động bằng Python, chạy skill trong sandbox, ghi trace và benchmark.

Các tài liệu kết quả thực nghiệm và phân tích nằm ngay ở root repo:

- `EVALUATION_AND_TUNING.md`
- `TUNING_METHODOLOGY.md`
- `TUNING_RESULTS.md`
- `DOCUMENTATION_INDEX.md`

## Cần chuẩn bị

- Python 3.10+
- Rust toolchain + Cargo
- Podman
- Một endpoint LLM kiểu OpenAI-compatible cho phần Python benchmark, hoặc Ollama local

## Cấu trúc nhanh

- `morale-rust/data/skills/`: corpus skill benign và malicious
- `morale-rust/src/`: logic audit và risk checker
- `pgdrsa-python/pgdrsa/`: engine benchmark, sandbox, receiver, judge
- `pgdrsa-python/scripts/`: script benchmark ngẫu nhiên và file kết quả mẫu

## Cách chạy Python benchmark

Vào thư mục Python trước khi chạy các lệnh dưới đây:

```bash
cd pgdrsa-python
python3 -m venv .venv
. .venv/bin/activate
pip install -e ".[dev]"
```

### 1. Chạy một skill đơn lẻ

```bash
. .venv/bin/activate
export PGDRSA_LLM_API_KEY='ollama'
export PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1'
export PGDRSA_LLM_MODEL='llama3.1:latest'
python -m pgdrsa.smoke --skill ../morale-rust/data/skills/benign/076-memory-note
```

Lệnh này chạy một skill trong sandbox, thu trace và in JSON kết quả ra terminal.

### 2. Chạy toàn bộ corpus

```bash
. .venv/bin/activate
python -m pgdrsa.run_all --results-file pgdrsa_results.jsonl
```

Tuỳ chọn hữu ích:

- `--limit N`: chỉ chạy N skill đầu tiên
- `--retry-errors`: chạy lại các skill từng lỗi
- `--only-bad BASELINE_JSONL`: chỉ chạy những skill lỗi hoặc false negative từ baseline
- `--image IMAGE`: override image sandbox

### 3. Chạy benchmark ngẫu nhiên

```bash
. .venv/bin/activate
python scripts/run_random_benchmark.py --count 10 --seed 123 --results-file scripts/pgdrsa_random_results.jsonl
```

Script này chọn ngẫu nhiên một tập skill từ corpus, chạy từng skill, rồi ghi từng dòng JSONL vào file kết quả. Sau đó script còn tính thêm metrics cơ bản và lưu kèm file `.metrics.json`.

### 4. Xem và phân tích kết quả

Các script phân tích đã có sẵn ở root repo:

```bash
python analyze_tuning_results.py
python compare_samples.py
```

Kết quả benchmark mẫu hiện được lưu trong `pgdrsa-python/scripts/pgdrsa_random_results.jsonl`.

## Cách chạy Rust auditor

```bash
cd morale-rust
cargo test
cargo run -- ../morale-rust/data/skills/benign/076-memory-note
```

Binary `morale` nhận một thư mục skill hoặc file `.zip`, rồi chạy audit và in báo cáo theo các flag CLI.

Một số flag hay dùng:

- `--lightweight`: bỏ qua database
- `--json`: xuất JSON
- `--summary`: chỉ in tóm tắt
- `--stats`: xem thống kê rủi ro
- `--enable_ai`: bật phân tích AI-powered nếu đã cấu hình API key

## Local Ollama

Nếu muốn chạy hoàn toàn local cho phần Python benchmark, cài và khởi động Ollama rồi export các biến môi trường sau:

```bash
brew install ollama
ollama serve
ollama pull llama3.1:latest

export PGDRSA_LLM_API_KEY='ollama'
export PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1'
export PGDRSA_LLM_MODEL='llama3.1:latest'
```

## Kết quả và báo cáo

Nếu bạn đang tìm phần mô tả kết quả thực nghiệm, thứ tự đọc khuyến nghị là:

1. `DOCUMENTATION_INDEX.md`
2. `PAPER_KNOWLEDGE_BASE.md`
3. `EVALUATION_AND_TUNING.md`
4. `TUNING_RESULTS.md`

## Troubleshooting nhanh

- Nếu gặp lỗi `ModuleNotFoundError: No module named 'openai'`, hãy chắc chắn đã cài đúng môi trường Python với `pip install -e ".[dev]"` trong `pgdrsa-python/`.
- Nếu benchmark không chạy được do podman/network, kiểm tra lại Podman đã sẵn sàng và image sandbox đã build.
- Nếu chạy skill nhưng không thấy trace như mong đợi, thử chạy một skill benign trước để kiểm tra pipeline end-to-end.
