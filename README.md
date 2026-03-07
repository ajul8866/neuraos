# NeuraOS

**The Agent Operating System — autonomous AI agents that work for you.**

NeuraOS is a production-grade Rust framework for building, deploying, and orchestrating
autonomous AI agents. It provides a complete operating system layer for AI: kernel,
runtime, memory, LLM routing, tool execution, security, and a REST/WebSocket API — all
in a single self-contained binary.

---

## Features

- **10 pre-built specialist agents** — researcher, coder, devops, data analyst, writer,
  secretary, security analyst, product manager, financial analyst, teacher
- **45 LLM providers** — OpenAI, Anthropic, Google, Groq, DeepSeek, Mistral, Ollama,
  and 38 more via a unified routing layer with Bayesian cost/quality optimisation
- **Multi-modal memory** — working context, episodic (SQLite), semantic (Qdrant vectors),
  and knowledge graph with automatic consolidation and importance decay
- **50+ tools** — bash, Python, HTTP, web search/scrape, filesystem, Git, SQL, email,
  Slack, code execution, and more with WASM sandboxing
- **Security shield** — prompt injection detection, jailbreak detection, PII redaction,
  data-guard policies, and cryptographic audit logging
- **OpenAI-compatible API** — drop-in replacement for `/v1/chat/completions` with
  streaming support; existing OpenAI clients work without modification
- **RBAC** — fine-grained per-agent capability control with approval gates for
  destructive operations
- **Observability** — structured tracing (OpenTelemetry), Prometheus metrics, Sentry

---

## Quick Start

### Prerequisites

- Rust 1.80+ (`curl https://sh.rustup.rs | sh`)
- Docker + Docker Compose (for the full stack)

### Run with Docker Compose

```bash
git clone https://github.com/neuraos/neuraos
cd neuraos

# Copy and configure environment
cp env.example .env
# Edit .env — at minimum set OPENAI_API_KEY or ANTHROPIC_API_KEY

# Start the full stack (NeuraOS + Qdrant + Redis + Postgres)
docker compose up -d

# Verify health
curl http://localhost:8080/health
```

### Build from Source

```bash
git clone https://github.com/neuraos/neuraos
cd neuraos

# Copy environment
cp env.example .env

# Build release binary
cargo build --release --bin neuraos

# Run
./target/release/neuraos --config config/default.toml
```

### CLI

```bash
# Build the CLI
cargo build --release --bin neura

# Check server status
./target/release/neura server status

# List available agents
./target/release/neura agent list

# Spawn the researcher agent
./target/release/neura agent spawn researcher

# Chat with an agent
./target/release/neura chat --agent researcher "What are the latest developments in AI safety?"

# Query agent memory
./target/release/neura memory query --agent researcher --text "AI safety" --limit 5
```

---

## Architecture

```
neuraos/
├── crates/
│   ├── neuraos-types      # Shared domain types (AgentManifest, ToolCapability, ...)
│   ├── neuraos-config     # TOML + env configuration loading
│   ├── neuraos-kernel     # Event bus, scheduler, planner, budget, circuit breaker, RBAC
│   ├── neuraos-memory     # Working, episodic, semantic, graph memory + consolidator
│   ├── neuraos-llm        # LLM router, 45 providers, Bayesian optimizer, cache, streaming
│   ├── neuraos-tools      # Tool executor, registry, WASM sandbox, 50+ built-in tools
│   ├── neuraos-runtime    # Agent loop (ReAct/PlanAndExecute), spawner, context, watcher
│   ├── neuraos-api        # Axum REST + WebSocket server, OpenAI-compat routes
│   ├── neuraos-shield     # Injection/jailbreak detection, data guard, audit log
│   ├── neuraos-agents     # Pre-built catalog: 10 specialist agent manifests
│   ├── neuraos-cli        # `neura` CLI binary
│   └── neuraos-bin        # `neuraos` server binary (wires everything together)
```

### Request Flow

```
Client
  │
  ▼
neuraos-api  (Axum HTTP/WS)
  │  auth middleware → rate limiter → shield (injection check)
  ▼
neuraos-kernel  (event bus → scheduler)
  │
  ▼
neuraos-runtime  (agent loop: ReAct / Plan-and-Execute)
  │         │
  │         ├── neuraos-llm   (route → provider → stream response)
  │         ├── neuraos-tools (sandbox → execute → result)
  │         └── neuraos-memory (retrieve context → store traces)
  │
  ▼
neuraos-shield  (audit every action)
```

---

## Configuration

Configuration is loaded from `config/default.toml` (or the path given to `--config`).
Every setting can be overridden via environment variable. See `env.example` for the
complete reference.

Key sections:

| Section     | Description                                         |
|-------------|-----------------------------------------------------|
| `[server]`  | HTTP port, TLS, workers, CORS                      |
| `[api]`     | Authentication, rate limits, OpenAI-compat path    |
| `[llm]`     | Default model, cache, Bayesian optimizer weights   |
| `[memory]`  | Capacity, embedding model, decay, SQLite path      |
| `[tools]`   | Enable/disable individual tools, sandbox settings  |
| `[kernel]`  | Event bus capacity, scheduler tick, circuit breaker|
| `[shield]`  | Injection detection, data guard, audit log         |
| `[agents]`  | Default budgets, pre-load catalog, approval gates  |
| `[telemetry]`| Log level/format, OTLP, Prometheus               |
| `[storage]` | Database URLs, data directories                    |

---

## API Reference

NeuraOS exposes a REST API with an OpenAI-compatible layer.

### Health

```
GET /health
```

### Agents

```
GET    /v1/agents              # list all running agents
POST   /v1/agents              # spawn a new agent
GET    /v1/agents/:id          # agent status
DELETE /v1/agents/:id          # kill agent
```

### Tasks

```
GET    /v1/tasks               # list all tasks
GET    /v1/tasks/:id           # task status + result
DELETE /v1/tasks/:id           # cancel task
```

### Memory

```
GET    /v1/memory/:agent_id?q=...&limit=10   # semantic search
DELETE /v1/memory/:agent_id                  # clear agent memory
```

### Tools

```
GET    /v1/tools               # list registered tools
POST   /v1/tools/execute       # execute a tool directly
```

### OpenAI-compatible

```
POST   /v1/chat/completions    # chat (streaming supported)
GET    /v1/models              # model list
```

### WebSocket

```
WS     /v1/ws                  # real-time event stream
```

---

## Pre-built Agents

| Slug               | Description                                        | Model                    |
|--------------------|----------------------------------------------------|--------------------------|
| `researcher`       | Deep web research, fact-checking, cited reports    | gpt-4o                   |
| `coder`            | Code generation, review, debugging, refactoring    | claude-3-5-sonnet        |
| `devops`           | CI/CD, Kubernetes, Terraform, monitoring           | gpt-4o                   |
| `data_analyst`     | CSV/SQL analysis, statistics, ML, visualisation    | gpt-4o                   |
| `writer`           | Articles, docs, copy, editing                      | claude-3-5-sonnet        |
| `secretary`        | Email drafting, scheduling, reminders              | gpt-4o-mini              |
| `security_analyst` | OWASP assessment, threat modelling, CVE scanning   | gpt-4o                   |
| `product_manager`  | PRDs, roadmaps, prioritisation, stakeholder comms  | claude-3-5-sonnet        |
| `financial_analyst`| DCF models, risk assessment, financial reports     | gpt-4o                   |
| `teacher`          | Concept explanation, curriculum, Socratic tutoring | gpt-4o                   |

Use `neuraos_agents::get_agent("researcher")` in Rust or `POST /v1/agents` via API.

---

## Security

- All agents run with declared capability sets — no tool can be called unless it is
  listed in `AgentManifest::capabilities`.
- The shield layer scans every prompt for injection patterns before LLM invocation.
- Agent manifests are Ed25519-signed; tampered manifests are rejected at load time.
- Human-in-the-loop approval gates can be set per agent and per tool category.
- All tool calls are recorded in a tamper-evident audit log.
- The server binary drops to a non-root user in Docker.

---

## Development

```bash
# Run tests
cargo test --workspace

# Check formatting
cargo fmt --check

# Clippy lints
cargo clippy --workspace -- -D warnings

# Build docs
cargo doc --workspace --no-deps --open

# Run the xtask helper
cargo xtask --help
```

---

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

---

## Contributing

Pull requests welcome! Please read `CONTRIBUTING.md` first. All contributions are
subject to the project's code of conduct.
