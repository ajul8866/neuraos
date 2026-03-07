# Changelog

All notable changes to NeuraOS are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
- Initial public release of NeuraOS v0.1.0

---

## [0.1.0] - 2026-03-08

### Added

#### Core framework
- `neuraos-types` — shared domain types: `AgentManifest`, `AgentId`, `AgentState`,
  `AgentMode`, `ReasoningStrategy`, `AgentBudget`, `ApprovalGates`, `ToolCapability`,
  `ToolDefinition`, `ToolCall`, `ToolResult`, `MemoryConfig`, `MemoryEntry`,
  `ModelPreference`, `LlmRequest`, `LlmResponse`, `LlmStreamChunk`, and 30+ more
- `neuraos-config` — TOML + environment variable configuration with full validation,
  section structs for server, API, LLM, memory, tools, kernel, shield, agents,
  telemetry, and storage

#### Kernel
- `neuraos-kernel` — agent operating system kernel:
  - Broadcast event bus (`tokio::sync::broadcast`) with typed `NeuraEvent` enum
  - Cron + one-shot scheduler with `cron` expression parsing
  - Hierarchical task planner with dependency resolution and critical path analysis
  - Per-agent budget tracker with hard limits on cost, tokens, tool calls, and duration
  - Circuit breaker (Closed → Open → Half-Open state machine) per service
  - Role-based access control (RBAC) with `Permission` enum and policy evaluation

#### Memory
- `neuraos-memory` — multi-modal memory subsystem:
  - In-process working memory store (`DashMap`-backed)
  - SQLite episodic memory with full-text search and importance decay
  - Qdrant-backed semantic (vector) memory with cosine similarity search
  - Knowledge graph with entity/edge storage and multi-hop traversal
  - Background consolidator: deduplication, decay, archival

#### LLM
- `neuraos-llm` — universal LLM routing layer:
  - 45 provider implementations: OpenAI, Anthropic, Google, Groq, DeepSeek, Mistral,
    Ollama, Together AI, Fireworks, Perplexity, Cohere, xAI, Azure, AWS Bedrock,
    Vertex AI, and 30 more
  - Bayesian model optimizer: selects best model per request by quality/cost/speed
  - Response cache with configurable TTL and semantic deduplication
  - First-class streaming with `LlmStreamChunk` typed enum
  - Automatic retry with exponential backoff and per-provider circuit breaking

#### Tools
- `neuraos-tools` — tool execution engine:
  - Sandboxed WASM + subprocess execution with resource limits
  - Tool registry with JSON Schema parameter validation
  - Built-in tools: `bash`, `python`, `http_get`, `http_post`, `web_search`,
    `web_scrape`, `read_file`, `write_file`, `delete_file`, `git_diff`, `sql_query`
  - Approval gate integration: dangerous tools require human confirmation
  - Per-call cost tracking and WASM fuel metering

#### Runtime
- `neuraos-runtime` — agent execution engine:
  - ReAct loop: Thought → Action → Observation with configurable max depth
  - Plan-and-Execute: LLM planner generates steps, executor runs them serially
  - Agent context management: prompt assembly, tool injection, history compression
  - Multi-agent spawner with parent→child orchestration
  - Watcher: health monitoring, budget enforcement, timeout handling

#### API
- `neuraos-api` — HTTP + WebSocket server (Axum 0.8):
  - Full REST API: `/v1/agents`, `/v1/tasks`, `/v1/memory`, `/v1/tools`
  - OpenAI-compatible `/v1/chat/completions` with SSE streaming
  - WebSocket hub at `/v1/ws` for real-time event delivery
  - Middleware: API key auth, rate limiting, request tracing, CORS
  - `/health` endpoint with subsystem status

#### Security
- `neuraos-shield` — security subsystem:
  - Prompt injection detector: 30+ pattern signatures, Unicode normalisation
  - Jailbreak detector: role-play bypass, privilege escalation patterns
  - Data guard: PII detection (email, phone, SSN, credit card, IP address)
  - Input sanitiser: HTML/script stripping, control character removal
  - Threat model: `ThreatLevel` classification with `ThreatAssessment` output
  - Audit logger: append-only log with SHA-256 chain integrity verification

#### Agents
- `neuraos-agents` — pre-built specialist catalog:
  - 10 production-ready `AgentManifest` instances with full system prompts,
    capability sets, budget constraints, and model preferences
  - `catalog()` — returns all 10 agents sorted alphabetically
  - `get_agent(name)` — look up agent by slug name
  - Agents: `researcher`, `coder`, `devops`, `data_analyst`, `writer`,
    `secretary`, `security_analyst`, `product_manager`, `financial_analyst`, `teacher`

#### CLI
- `neuraos-cli` (`neura` binary) — full-featured CLI:
  - `agent list | spawn | status | kill`
  - `task list | status | cancel`
  - `memory query | clear`
  - `tool list | exec`
  - `chat` — interactive chat with streaming
  - `server start | status`
  - `logs` — SSE log streaming
  - `config` — show active configuration
  - Global flags: `--server`, `--api-key`, `--output` (table/json/plain), `--verbose`

#### Server binary
- `neuraos-bin` (`neuraos` binary) — server entrypoint:
  - Wires all subsystems together in a clean startup sequence
  - Graceful shutdown on SIGINT/SIGTERM
  - Multi-threaded Tokio runtime (configurable worker count)
  - Structured logging (human or JSON) via `tracing-subscriber`
  - ASCII banner with version and boot time on startup

#### Infrastructure
- `Dockerfile` — multi-stage build with `cargo-chef` dependency caching;
  minimal `debian:bookworm-slim` runtime image; non-root `neuraos` user
- `docker-compose.yml` — full stack: NeuraOS + Qdrant + Redis + PostgreSQL
  with health checks and named volumes
- `config/default.toml` — fully-documented default configuration
- `env.example` — complete environment variable reference with descriptions
- `README.md` — quick start, architecture diagram, API reference, agent catalog

### Technical details
- Rust 1.80, edition 2021
- Async runtime: Tokio 1.x (multi-thread, full features)
- HTTP server: Axum 0.8 with Tower middleware
- Serialisation: serde + serde_json (preserve_order, raw_value)
- Error handling: thiserror + anyhow
- Concurrency: DashMap 6, crossbeam, parking_lot
- All crates: `#![forbid(unsafe_code)]`

---

[Unreleased]: https://github.com/neuraos/neuraos/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/neuraos/neuraos/releases/tag/v0.1.0
