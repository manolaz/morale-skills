# Engineering and Architecture Decisions - Morale Skills Auditor

This document details the key engineering and architectural decisions made during the design, development, and refactoring of the **Morale Skills Auditor** tool. These choices represent the trade-offs, constraints, and solutions adopted to ensure a high-performance, secure, and extensible AI skill auditing engine.

---

## 1. Storage Engine Choice: SurrealDB In-Memory (`Mem`)

### Context
The auditor requires a database to store skills, parsed findings, and detailed audit trails. It needs to support:
1. Fast, sub-millisecond writes and lookups during scans.
2. Robust querying capabilities (filtering by severity, risk type, timestamp, or full-text query matching).
3. Relation-based navigation (associating findings and logs to a scanned skill).
4. No dependencies on external, heavy database servers (like PostgreSQL or MySQL) to maintain a zero-setup CLI user experience.

### Decision
We chose **SurrealDB** running with the **local in-memory storage engine (`Mem`)**.

```rust
let db = Surreal::new::<Mem>(()).await?;
```

### Rationale
* **Embedded & Serverless:** SurrealDB can be run embedded directly within the Rust binary, meaning the user does not need to install or run any external database engines.
* **Rich Querying & Schema Schemafulness:** Unlike simpler key-value stores (such as sled or rocksdb), SurrealDB supports a full SQL-like query language, multi-table joins, and graph relations while maintaining schema rigidity via `SCHEMAFULL` table definitions.
* **Performance:** Running in `Mem` mode keeps all data in RAM, resulting in ultra-fast operational latencies well-suited for a CLI utility.

### Trade-offs
* **Non-persistence across sessions:** By default, in-memory databases lose all records when the process exits. To mitigate this, standard CLI configurations can run the memory db but export reports to JSON/TOML, and future iterations can easily swap `Mem` with a file-based rocksdb/speckled driver via `config.toml` modifications without altering any query logic.

---

## 2. SurrealDB Connection Trait Bounds & Deserialization Refactoring

### Context
During implementation, compiling the Rust client for SurrealDB produced complex trait bound errors. Specifically, the database handle was typed as `Surreal<Mem>`, which failed when standard connection methods expected the underlying connection driver `Db` rather than the config engine `Mem`.

### Decision
We refactored `Database` and all query submodules to explicitly decouple the initialization config from the database connection type.

1. **Decoupled Handle Typing:**
   ```rust
   // src/database/connection.rs
   pub struct Database {
       pub db: Surreal<Db>, // Connection driver Db instead of Mem config
   }
   ```
2. **Precise Type Annotations during Deserialization:**
   To help the Rust compiler and serde-deserializer resolve types properly on complex queries, we added explicit annotations to the database actions instead of relying entirely on type inference:
   ```rust
   let created_skill: AISkill = self.db.create("ai_skills")
       .content(skill)
       .await?
       .into_iter()
       .next()
       .ok_or(anyhow::anyhow!("Failed to create skill record"))?;
   ```

### Rationale
This architecture ensures that compile-time checks are robust, type-coercion bugs are avoided, and the connection handle remains generic enough to support switching local backend engines (e.g. RocksDB or Indb) in the future.

---

## 3. Modular & Trait-Driven Checker Architecture

### Context
Initially, risk checks were handled via monolithic, hardcoded helper functions. This design made it difficult to introduce new security checks, test checkers in isolation, or dynamically enable/disable checks based on user preferences.

### Decision
We introduced a modular, trait-driven architecture leveraging Rust's object safety and async capabilities.

1. **The `RiskChecker` Trait:**
   ```rust
   #[async_trait::async_trait]
   pub trait RiskChecker {
       fn name(&self) -> &'static str;
       async fn check(&self, path: &str) -> Result<Vec<RiskFinding>>;
       fn risk_category(&self) -> &'static str;
   }
   ```
2. **The `RiskCheckerRegistry`:**
   A registry acts as a single point of orchestration, housing all checker instances.
   ```rust
   pub struct RiskCheckerRegistry {
       checkers: Vec<Arc<dyn RiskChecker + Send + Sync>>,
   }
   ```

### Rationale
* **Extensibility (Open-Closed Principle):** Adding a new checker (like the recently integrated `ToolsPermissionsChecker`) only requires implementing the `RiskChecker` trait and adding it to the registry vector. No existing main orchestration logic needs to change.
* **Separation of Concerns:** Each vulnerability type has its own isolated logic file (e.g., `src/risks/network.rs`, `src/risks/prompt_injection.rs`), facilitating testing and code reviews.

---

## 4. Hybrid Static/Heuristic and AI-Powered Scanning

### Context
Static heuristic checks (looking for hardcoded API keys, unpinned dependencies, specific system words) are incredibly fast but often fail to evaluate semantic risks (such as advanced, indirect prompt injection or context poisoning attacks).

### Decision
We designed a hybrid auditing pipeline:
1. **Static / Heuristic Checks:** Run by default across all files. They perform lightning-fast regex/sub-string scanning.
2. **AI-Powered Semantic Analysis (Optional):** Triggered via `--enable-ai`. Code snippets are extracted and dispatched to state-of-the-art LLMs (Claude, Gemini, OpenAI, Qwen).

### Robustness & Fallback Design
External API requests can fail or return unexpected markdown text instead of structured JSON. To handle this:
* We implement file size filtering (only sending text files containing between `1` and `10,000` characters to prevent API rate-limit or cost explosions).
* We implement a fallback parser. If the model's output cannot be decoded into structured JSON finding objects, we gracefully catch the error and serialize the entire raw text response as standard `evidence` under `RiskType::AIBased` instead of failing the entire audit.

---

## 5. Async Orchestration and CLI User Experience

### Context
Performing complex file walks, heuristic regex checks, and optional external network-bound API calls can take time. A poorly designed CLI can feel sluggish, non-responsive, or opaque.

### Decision
1. **Tokio Async Runtime:** The entire application runs on `#[tokio::main]` utilizing async tasks to prevent thread-blocking during heavy file system I/O or network API requests.
2. **Rich CLI Progress Indicators:** We integrated `indicatif` with custom, non-blocking spinner templates.
   - When running static checks, a spinner indicates active processing.
   - When executing AI analysis, a clear progress bar displays `[processed_files]/[total_files]` along with estimated remaining time.
3. **Structured Verbose Logging:** Standard output outputs a clean, colorful, and human-readable security report grouped by severity (Critical first). Behind the scenes, detailed trace levels (`debug`, `warn`, `error`) are captured and exposed only when running with the `-v` / `--verbose` flag.
