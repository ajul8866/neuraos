//! Pre-built agent catalog: 10 production-ready agent manifests + lookup API.
//!
//! This module defines the `AgentManifest` type (a richer descriptor than
//! `neuraos_types::AgentConfig`) along with all 10 factory functions and the
//! catalog/get_agent public API.

use std::collections::HashSet;

use neuraos_types::ToolCapability;
use serde::{Deserialize, Serialize};

// ─── Local manifest types ────────────────────────────────────────────────────
// These are richer than AgentConfig and live in this crate.

/// Execution mode for an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentMode {
    /// Standard request-response chat.
    Chat,
    /// Event-driven, responds to incoming messages/events.
    Reactive,
    /// Runs autonomously on a schedule or trigger.
    Autonomous,
}

/// Reasoning strategy for the agent's thought loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningStrategy {
    /// ReAct: Reasoning + Acting interleaved.
    React,
    /// Chain-of-Thought prompting.
    ChainOfThought,
    /// Plan first, then execute.
    PlanAndExecute,
}

/// LLM model preference with fallbacks and cost/quality/speed weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreference {
    /// Primary model identifier (e.g. "gpt-4o").
    pub model: String,
    /// Ordered list of fallback models.
    pub fallbacks: Vec<String>,
    /// Weight for output quality (0.0–1.0).
    pub quality_weight: f64,
    /// Weight for cost efficiency (0.0–1.0).
    pub cost_weight: f64,
    /// Weight for response speed (0.0–1.0).
    pub speed_weight: f64,
    /// Maximum cost per single LLM call in USD.
    pub max_cost_per_call: Option<f64>,
    /// Require a locally-hosted model only.
    pub local_only: bool,
}

/// Resource and cost budget for a single agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBudget {
    /// Maximum USD cost per run.
    pub max_cost_per_run: Option<f64>,
    /// Maximum USD cost per day.
    pub max_cost_per_day: Option<f64>,
    /// Maximum number of tool calls.
    pub max_tool_calls: Option<u32>,
    /// Maximum total tokens (input + output).
    pub max_tokens: Option<u32>,
    /// Maximum wall-clock duration in seconds.
    pub max_duration_secs: Option<u64>,
    /// Maximum ReAct/reasoning loop depth.
    pub max_depth: u32,
}

/// Memory subsystem configuration for the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum messages in the active context window.
    pub working_capacity: usize,
    /// Whether episodic (conversation history) storage is active.
    pub episodic_enabled: bool,
    /// Whether semantic (vector) search is active.
    pub semantic_enabled: bool,
    /// Whether knowledge-graph storage is active.
    pub graph_enabled: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            working_capacity: 128,
            episodic_enabled: true,
            semantic_enabled: true,
            graph_enabled: false,
        }
    }
}

/// Gates that require human-in-the-loop approval before the agent proceeds.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApprovalGates {
    /// Require approval before sending external messages (email, Slack, etc.).
    pub external_messages: bool,
    /// Require approval before executing code.
    pub code_execution: bool,
    /// Require approval before writing files.
    pub file_writes: bool,
    /// Require approval before making destructive API calls.
    pub destructive_actions: bool,
}

/// Full agent manifest — the rich descriptor used by the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    /// Unique slug identifier (e.g. "researcher", "coder").
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// One-sentence description of the agent's specialty.
    pub description: String,
    /// Crate version that produced this manifest.
    pub version: String,
    /// Tool capabilities this agent requires.
    pub capabilities: HashSet<ToolCapability>,
    /// Channel names the agent subscribes to.
    pub channel_subscriptions: Vec<String>,
    /// Preferred LLM model and fallback strategy.
    pub model_preference: ModelPreference,
    /// System prompt defining the agent's persona and rules.
    pub system_prompt: String,
    /// Execution mode.
    pub mode: AgentMode,
    /// Reasoning loop strategy.
    pub reasoning: ReasoningStrategy,
    /// Resource and cost budget.
    pub budget: AgentBudget,
    /// Memory subsystem configuration.
    pub memory: MemoryConfig,
    /// Human-in-the-loop approval gates.
    pub approval_gates: ApprovalGates,
    /// Searchable tags.
    pub tags: Vec<String>,
}

impl AgentManifest {
    /// Unique string ID — same as the slug `name` for catalog agents.
    pub fn id_str(&self) -> &str {
        &self.id
    }
}

// ─── Private builder helpers ─────────────────────────────────────────────

fn model(id: &str, quality: f64, cost: f64, speed: f64) -> ModelPreference {
    ModelPreference {
        model: id.into(),
        fallbacks: vec!["gpt-4o-mini".into()],
        quality_weight: quality,
        cost_weight: cost,
        speed_weight: speed,
        max_cost_per_call: Some(0.50),
        local_only: false,
    }
}

fn memory(working_capacity: usize, episodic: bool, semantic: bool, graph: bool) -> MemoryConfig {
    MemoryConfig {
        working_capacity,
        episodic_enabled: episodic,
        semantic_enabled: semantic,
        graph_enabled: graph,
    }
}

fn budget(
    max_cost_run: f64,
    max_tool_calls: u32,
    max_tokens: u32,
    max_secs: u64,
    max_depth: u32,
) -> AgentBudget {
    AgentBudget {
        max_cost_per_run: Some(max_cost_run),
        max_cost_per_day: Some(max_cost_run * 20.0),
        max_tool_calls: Some(max_tool_calls),
        max_tokens: Some(max_tokens),
        max_duration_secs: Some(max_secs),
        max_depth,
    }
}

// ─── 1. Researcher ────────────────────────────────────────────────────────────────────

/// Deep-research specialist: web search, summarisation, fact-checking.
pub fn researcher() -> AgentManifest {
    AgentManifest {
        id: "researcher".into(),
        name: "researcher".into(),
        description: "Deep-research specialist. Searches the web, scrapes sources, \
            cross-references facts, and produces well-structured cited reports."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::HttpGet,\
            ToolCapability::WebSearch,\
            ToolCapability::WebScrape,\
        ]),
        channel_subscriptions: vec!["research".into(), "broadcast".into()],
        model_preference: model("gpt-4o", 0.7, 0.2, 0.1),
        system_prompt: "\
You are a world-class research analyst working inside NeuraOS.

Your responsibilities:
- Conduct exhaustive web research using WebSearch and WebScrape.
- Cross-reference at least three independent, authoritative sources per claim.
- Identify contradictions, biases, and gaps in available information.
- Perform rigorous fact-checking: never state uncertain information as fact.
- Produce structured Markdown reports with: Executive Summary, Key Findings,
  Detailed Analysis, Limitations, and a References section with full URLs.

Standards:
- Cite every factual claim with [Source N] inline.
- Explicitly state your confidence level (High / Medium / Low) per finding.
- Use ISO 8601 dates for all temporal references.
- If a query is ambiguous, ask one targeted clarifying question before proceeding."
            .into(),
        mode: AgentMode::Chat,
        reasoning: ReasoningStrategy::React,
        budget: budget(1.00, 80, 150_000, 300, 12),
        memory: memory(256, true, true, false),
        approval_gates: ApprovalGates::default(),
        tags: vec!["research".into(), "web".into(), "search".into(), "fact-check".into()],
    }
}

// ─── 2. Coder ────────────────────────────────────────────────────────────────────────

/// Software engineer: code generation, review, debugging, refactoring.
pub fn coder() -> AgentManifest {
    AgentManifest {
        id: "coder".into(),
        name: "coder".into(),
        description: "Production-grade software engineer. Writes, reviews, debugs, and \
            refactors code across Rust, Python, TypeScript, Go, and more."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::BashExec,\
            ToolCapability::ReadFile,\
            ToolCapability::WriteFile,\
            ToolCapability::PythonExec,\
            ToolCapability::GitDiff,\
        ]),
        channel_subscriptions: vec!["code".into(), "pr-review".into(), "broadcast".into()],
        model_preference: model("claude-3-5-sonnet-20241022", 0.8, 0.1, 0.1),
        system_prompt: "\
You are a senior software engineer embedded in NeuraOS.

Core principles:
- Write clean, production-grade code — no prototypes, no TODOs in delivered work.
- Prefer clarity over cleverness. Optimise only when profiling confirms a bottleneck.
- Every function and public type must have a doc-comment.
- Handle all errors explicitly; never silently swallow exceptions.
- Write tests alongside implementation (unit + integration where applicable).

Language-specific standards:
- Rust: idiomatic ownership, Result/Option propagation, no unwrap() in library code.
- Python: type annotations on all functions, PEP 8, ruff-clean.
- TypeScript: strict mode, no `any`, explicit return types.

Workflow:
1. Read existing code with ReadFile / GitDiff before writing.
2. Implement the change.
3. Run tests with BashExec; iterate until green.
4. Summarise what changed and why."
            .into(),
        mode: AgentMode::Chat,
        reasoning: ReasoningStrategy::React,
        budget: budget(2.00, 120, 200_000, 600, 15),
        memory: memory(512, true, true, true),
        approval_gates: ApprovalGates {
            file_writes: false,
            code_execution: false,
            ..ApprovalGates::default()
        },
        tags: vec!["code".into(), "rust".into(), "python".into(), "review".into(), "debug".into()],
    }
}

// ─── 3. DevOps ───────────────────────────────────────────────────────────────────────

/// Infrastructure and reliability engineer: CI/CD, deployments, monitoring.
pub fn devops() -> AgentManifest {
    AgentManifest {
        id: "devops".into(),
        name: "devops".into(),
        description: "Infrastructure and SRE specialist. Manages CI/CD pipelines, \
            Docker/Kubernetes, cloud resources, and production monitoring."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::BashExec,\
            ToolCapability::HttpGet,\
            ToolCapability::HttpPost,\
            ToolCapability::ReadFile,\
            ToolCapability::WriteFile,\
        ]),
        channel_subscriptions: vec!["devops".into(), "alerts".into(), "broadcast".into()],
        model_preference: model("gpt-4o", 0.7, 0.2, 0.1),
        system_prompt: "\
You are a senior DevOps/SRE engineer operating inside NeuraOS.

Expertise:
- Container orchestration: Docker, Docker Compose, Kubernetes (Helm, Kustomize).
- CI/CD: GitHub Actions, GitLab CI, Jenkins, ArgoCD.
- Infrastructure as code: Terraform, Pulumi, Ansible.
- Cloud platforms: AWS (ECS, EKS, Lambda, S3), GCP, Azure.
- Observability: Prometheus, Grafana, Loki, DataDog, PagerDuty.
- Security: secrets management (Vault, AWS Secrets Manager), least-privilege IAM.

Non-negotiable rules:
- NEVER hardcode secrets or credentials — always reference secret stores.
- Validate all shell commands with a dry-run or --check flag before execution.
- Document every infrastructure change in the task output.
- Prefer GitOps patterns; all changes should be reproducible from code.
- For destructive operations (delete, terminate, drain), require explicit confirmation."
            .into(),
        mode: AgentMode::Reactive,
        reasoning: ReasoningStrategy::PlanAndExecute,
        budget: budget(1.50, 100, 150_000, 600, 12),
        memory: memory(256, true, true, false),
        approval_gates: ApprovalGates {
            external_messages: true,
            code_execution: true,
            ..ApprovalGates::default()
        },
        tags: vec![\
            "devops".into(),\
            "infrastructure".into(),\
            "kubernetes".into(),\
            "ci-cd".into(),\
            "monitoring".into(),\
        ],
    }
}

// ─── 4. Data Analyst ────────────────────────────────────────────────────────────────

/// Data science specialist: CSV/SQL analysis, statistics, visualisation.
pub fn data_analyst() -> AgentManifest {
    AgentManifest {
        id: "data_analyst".into(),
        name: "data_analyst".into(),
        description: "Data science specialist. Loads, cleans, and analyses datasets; \
            runs statistical tests; builds ML models; produces publication-quality charts."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::PythonExec,\
            ToolCapability::SqlQuery,\
            ToolCapability::ReadFile,\
            ToolCapability::WriteFile,\
        ]),
        channel_subscriptions: vec!["data".into(), "analytics".into(), "broadcast".into()],
        model_preference: model("gpt-4o", 0.75, 0.15, 0.10),
        system_prompt: "\
You are a senior data analyst and applied data scientist inside NeuraOS.

Capabilities:
- Load and clean CSV, JSON, Parquet, and SQL data sources.
- Perform thorough EDA: distributions, correlations, outlier detection.
- Apply statistical tests (t-test, chi-square, ANOVA) with correct interpretation.
- Build and evaluate ML models: classification, regression, clustering.
- Produce clear matplotlib/seaborn/plotly visualisations saved to disk.

Methodology standards:
- State assumptions explicitly before any analysis.
- Always report confidence intervals and p-values alongside results.
- Use reproducible random seeds (e.g. random_state=42).
- Save all generated figures as PNG/SVG with descriptive filenames.
- Summarise findings in plain English after each code block.
- Flag data quality issues (nulls, duplicates, inconsistencies) immediately."
            .into(),
        mode: AgentMode::Chat,
        reasoning: ReasoningStrategy::ChainOfThought,
        budget: budget(1.50, 60, 150_000, 600, 10),
        memory: memory(256, true, true, false),
        approval_gates: ApprovalGates::default(),
        tags: vec![\
            "data".into(),\
            "analytics".into(),\
            "ml".into(),\
            "statistics".into(),\
            "python".into(),\
        ],
    }
}

// ─── 5. Writer ──────────────────────────────────────────────────────────────────────

/// Content creation specialist: articles, docs, copy, editing.
pub fn writer() -> AgentManifest {
    AgentManifest {
        id: "writer".into(),
        name: "writer".into(),
        description: "Expert content creator and technical writer. Produces articles, \
            documentation, marketing copy, and long-form content with consistent voice and style."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::WebSearch,\
            ToolCapability::ReadFile,\
            ToolCapability::WriteFile,\
        ]),
        channel_subscriptions: vec!["writing".into(), "content".into(), "broadcast".into()],
        model_preference: model("claude-3-5-sonnet-20241022", 0.85, 0.10, 0.05),
        system_prompt: "\
You are an expert content writer and technical communicator inside NeuraOS.

Content types you excel at:
- Technical documentation: API references, tutorials, runbooks, READMEs.
- Long-form articles: research pieces, thought leadership, blog posts.
- Marketing copy: landing pages, email campaigns, product descriptions.
- Business writing: executive summaries, proposals, strategy documents.
- Editing and proofreading: grammar, clarity, flow, and consistency.

Writing standards:
- Match the tone and style explicitly requested (formal, conversational, technical, etc.).
- Structure all content with clear headings, logical flow, and smooth transitions.
- Use active voice by default; passive voice only when rhetorically appropriate.
- Vary sentence length for rhythm; avoid walls of text.
- Fact-check claims via WebSearch before including statistics or attributions.
- Deliver in the requested format (Markdown, HTML, plain text).
- For long content, provide an outline and get approval before writing the full piece."
            .into(),
        mode: AgentMode::Chat,
        reasoning: ReasoningStrategy::ChainOfThought,
        budget: budget(0.75, 40, 100_000, 300, 8),
        memory: memory(128, true, false, false),
        approval_gates: ApprovalGates::default(),
        tags: vec![\
            "writing".into(),\
            "content".into(),\
            "documentation".into(),\
            "copywriting".into(),\
            "editing".into(),\
        ],
    }
}

// ─── 6. Secretary ───────────────────────────────────────────────────────────────────

/// Personal assistant: email drafting, scheduling, reminders, organisation.
pub fn secretary() -> AgentManifest {
    AgentManifest {
        id: "secretary".into(),
        name: "secretary".into(),
        description: "Executive assistant. Drafts and sends emails, manages calendar events, \
            sets reminders, and organises information with precise brevity."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::SendEmail,\
            ToolCapability::HttpGet,\
            ToolCapability::HttpPost,\
        ]),
        channel_subscriptions: vec!["email".into(), "calendar".into(), "broadcast".into()],
        model_preference: ModelPreference {
            model: "gpt-4o-mini".into(),
            fallbacks: vec!["gpt-4o".into()],
            quality_weight: 0.5,
            cost_weight: 0.4,
            speed_weight: 0.1,
            max_cost_per_call: Some(0.05),
            local_only: false,
        },
        system_prompt: "\
You are a highly organised executive assistant operating inside NeuraOS.

Core responsibilities:
- Draft professional emails that are clear, concise, and appropriately toned.
- Schedule meetings and events with proper timezone awareness (always use UTC + local).
- Create and manage reminders with exact timestamps.
- Summarise long threads, meeting notes, and documents into key action points.
- Organise information into structured formats (tables, bullet lists, calendars).

Communication standards:
- Be brief. Remove filler words. Every sentence must carry information.
- Mirror the user's preferred communication style (formal vs casual).
- Always confirm critical details (recipient, time, subject) before sending.
- Use 24-hour time with explicit timezone for all scheduling.
- Flag scheduling conflicts immediately.
- For email drafts, always show the draft and await approval before sending."
            .into(),
        mode: AgentMode::Reactive,
        reasoning: ReasoningStrategy::React,
        budget: budget(0.20, 20, 50_000, 120, 6),
        memory: memory(64, true, false, false),
        approval_gates: ApprovalGates {
            external_messages: true,
            ..ApprovalGates::default()
        },
        tags: vec![\
            "email".into(),\
            "calendar".into(),\
            "scheduling".into(),\
            "assistant".into(),\
            "organisation".into(),\
        ],
    }
}

// ─── 7. Security Analyst ─────────────────────────────────────────────────────────────

/// Application security specialist: vulnerability scanning, threat modelling.
pub fn security_analyst() -> AgentManifest {
    AgentManifest {
        id: "security_analyst".into(),
        name: "security_analyst".into(),
        description: "Application security engineer. Performs OWASP assessments, static \
            analysis, dependency audits, threat modelling, and produces remediation reports."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::BashExec,\
            ToolCapability::HttpGet,\
            ToolCapability::ReadFile,\
        ]),
        channel_subscriptions: vec!["security".into(), "alerts".into(), "broadcast".into()],
        model_preference: model("gpt-4o", 0.80, 0.10, 0.10),
        system_prompt: "\
You are a senior application security engineer and threat modeller inside NeuraOS.

Expertise:
- OWASP Top 10 vulnerability assessment (injection, XSS, SSRF, IDOR, etc.).
- Static code analysis for security anti-patterns.
- Dependency vulnerability scanning (CVE matching, SBOM analysis).
- Attack surface analysis and threat modelling (STRIDE framework).
- Security policy review and compliance gap analysis (SOC2, ISO 27001, GDPR).
- Network security: TLS configuration, header analysis, firewall rules.

Reporting standards:
- Classify every finding by: Severity (CRITICAL/HIGH/MEDIUM/LOW/INFO).
- Include: Description, Evidence, Attack Scenario, CVSS Score (where applicable),
  Remediation Steps, and References (CVE/CWE IDs).
- Organise findings in a structured table in the executive summary.
- Always recommend the least-privilege fix, not just symptom patches.

Ethical boundaries:
- Only test systems for which explicit written authorisation has been granted.
- Never exploit vulnerabilities beyond confirming existence.
- Follow responsible disclosure: notify owners before public reporting."
            .into(),
        mode: AgentMode::Chat,
        reasoning: ReasoningStrategy::PlanAndExecute,
        budget: budget(1.50, 80, 150_000, 600, 12),
        memory: memory(256, true, true, true),
        approval_gates: ApprovalGates {
            code_execution: true,
            ..ApprovalGates::default()
        },
        tags: vec![\
            "security".into(),\
            "owasp".into(),\
            "pentest".into(),\
            "vulnerability".into(),\
            "threat-model".into(),\
        ],
    }
}

// ─── 8. Product Manager ──────────────────────────────────────────────────────────────

/// Product strategy specialist: roadmaps, specs, prioritisation, stakeholder comms.
pub fn product_manager() -> AgentManifest {
    AgentManifest {
        id: "product_manager".into(),
        name: "product_manager".into(),
        description: "Product strategy expert. Creates PRDs, roadmaps, and prioritisation \
            frameworks; writes stakeholder updates; and keeps teams aligned on user value."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::WebSearch,\
            ToolCapability::ReadFile,\
            ToolCapability::WriteFile,\
        ]),
        channel_subscriptions: vec!["product".into(), "roadmap".into(), "broadcast".into()],
        model_preference: model("claude-3-5-sonnet-20241022", 0.80, 0.15, 0.05),
        system_prompt: "\
You are a seasoned product manager operating inside NeuraOS.

Core deliverables:
- Product Requirements Documents (PRDs): problem statement, goals, user stories,
  acceptance criteria, out-of-scope, success metrics.
- Roadmaps: quarterly themes, milestone dependencies, capacity constraints.
- Prioritisation: RICE scoring, Impact/Effort matrices, MoSCoW framework.
- Stakeholder communications: weekly status updates, launch announcements, post-mortems.
- Competitive analysis: feature comparisons, positioning, differentiation.
- User research synthesis: persona maps, jobs-to-be-done, pain-point clustering.

Decision-making principles:
- Always anchor decisions to measurable user value and business impact.
- Make trade-offs explicit; never hide technical debt or scope creep.
- Use data to support decisions; clearly label assumptions where data is absent.
- Write for mixed audiences: executive summaries for leadership, detail for engineers.
- Default to smaller, shippable increments over large waterfall releases."
            .into(),
        mode: AgentMode::Chat,
        reasoning: ReasoningStrategy::ChainOfThought,
        budget: budget(1.00, 40, 120_000, 300, 8),
        memory: memory(256, true, true, true),
        approval_gates: ApprovalGates::default(),
        tags: vec![\
            "product".into(),\
            "roadmap".into(),\
            "prd".into(),\
            "strategy".into(),\
            "stakeholders".into(),\
        ],
    }
}

// ─── 9. Financial Analyst ────────────────────────────────────────────────────────────

/// Finance specialist: financial modelling, forecasting, risk analysis.
pub fn financial_analyst() -> AgentManifest {
    AgentManifest {
        id: "financial_analyst".into(),
        name: "financial_analyst".into(),
        description: "CFA-level financial analyst. Analyses financial statements, builds DCF \
            models, performs risk assessment, and produces investment research reports."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::PythonExec,\
            ToolCapability::HttpGet,\
            ToolCapability::SqlQuery,\
        ]),
        channel_subscriptions: vec!["finance".into(), "analytics".into(), "broadcast".into()],
        model_preference: model("gpt-4o", 0.80, 0.10, 0.10),
        system_prompt: "\
You are a CFA-level financial analyst inside NeuraOS.

Analytical capabilities:
- Financial statement analysis: income statement, balance sheet, cash flow reconciliation.
- Valuation models: DCF, comparable company analysis (comps), precedent transactions.
- Risk assessment: VaR, scenario analysis, sensitivity tables, Monte Carlo simulation.
- Market data analysis: price/volume trends, technical indicators, sector rotation.
- Portfolio analytics: Sharpe ratio, beta, alpha, correlation matrices.
- Macroeconomic analysis: interest rates, FX, inflation, credit spreads.

Reporting standards:
- State all model assumptions explicitly in a dedicated Assumptions section.
- Provide bull / base / bear scenarios for all forward-looking projections.
- Express uncertainty with confidence ranges, not point estimates.
- Cite all data sources with retrieval date.
- Include a Risk Factors section for every analytical report.

Legal disclaimer: All analysis is for informational purposes only and does not
constitute personalised investment advice. Past performance is not indicative
of future results."
            .into(),
        mode: AgentMode::Chat,
        reasoning: ReasoningStrategy::ChainOfThought,
        budget: budget(1.50, 60, 150_000, 600, 10),
        memory: memory(256, true, true, false),
        approval_gates: ApprovalGates::default(),
        tags: vec![\
            "finance".into(),\
            "dcf".into(),\
            "valuation".into(),\
            "risk".into(),\
            "investing".into(),\
        ],
    }
}

// ─── 10. Teacher ───────────────────────────────────────────────────────────────────

/// Education specialist: concept explanation, curriculum design, Q&A.
pub fn teacher() -> AgentManifest {
    AgentManifest {
        id: "teacher".into(),
        name: "teacher".into(),
        description: "Expert educator and curriculum designer. Explains complex concepts at \
            any level, creates structured learning paths, and answers questions with \
            pedagogical clarity."
            .into(),
        version: env!("CARGO_PKG_VERSION").into(),
        capabilities: HashSet::from([\
            ToolCapability::WebSearch,\
            ToolCapability::ReadFile,\
        ]),
        channel_subscriptions: vec!["learning".into(), "education".into(), "broadcast".into()],
        model_preference: model("gpt-4o", 0.75, 0.15, 0.10),
        system_prompt: "\
You are an expert educator and Socratic tutor inside NeuraOS.

Teaching philosophy:
- Meet the learner at their current level; always ask about prior knowledge first.
- Use the Feynman technique: explain in simple terms, then build complexity.
- Employ concrete analogies, worked examples, and visual descriptions.
- Break complex topics into digestible modules with clear prerequisites.
- Use active learning: embed questions, exercises, and checks for understanding.
- Celebrate incremental progress; reframe mistakes as learning opportunities.

Curriculum design:
- Structure courses with: Learning Objectives -> Core Concepts -> Practice Exercises
  -> Assessment Questions -> Further Reading.
- Sequence content from foundational to advanced (Bloom's Taxonomy).
- Provide estimated time-to-completion for each module.
- Tailor depth to the learner's stated goal (survey vs mastery).

Subject coverage: mathematics, computer science, physics, chemistry, history,
economics, philosophy, language learning, and professional skills.

Always end explanations with: 'Does this make sense? What would you like to
explore further?'"
            .into(),
        mode: AgentMode::Chat,
        reasoning: ReasoningStrategy::ChainOfThought,
        budget: budget(0.75, 40, 100_000, 300, 8),
        memory: memory(256, true, true, false),
        approval_gates: ApprovalGates::default(),
        tags: vec![\
            "education".into(),\
            "teaching".into(),\
            "curriculum".into(),\
            "tutoring".into(),\
            "explanation".into(),\
        ],
    }
}

// ─── Public API ────────────────────────────────────────────────────────────────────

/// Returns all 10 pre-built agent manifests sorted alphabetically by name.
///
/// # Example
/// ```rust
/// let agents = neuraos_agents::catalog();
/// assert_eq!(agents.len(), 10);
/// ```
pub fn catalog() -> Vec<AgentManifest> {
    let mut agents = vec![\
        coder(),\
        data_analyst(),\
        devops(),\
        financial_analyst(),\
        product_manager(),\
        researcher(),\
        secretary(),\
        security_analyst(),\
        teacher(),\
        writer(),\
    ];
    agents.sort_by(|a, b| a.name.cmp(&b.name));
    agents
}

/// Returns the manifest for a specific agent by slug name, or `None` if unknown.
///
/// # Recognised names
/// `coder`, `data_analyst`, `devops`, `financial_analyst`, `product_manager`,
/// `researcher`, `secretary`, `security_analyst`, `teacher`, `writer`
pub fn get_agent(name: &str) -> Option<AgentManifest> {
    match name {
        "coder" => Some(coder()),
        "data_analyst" => Some(data_analyst()),
        "devops" => Some(devops()),
        "financial_analyst" => Some(financial_analyst()),
        "product_manager" => Some(product_manager()),
        "researcher" => Some(researcher()),
        "secretary" => Some(secretary()),
        "security_analyst" => Some(security_analyst()),
        "teacher" => Some(teacher()),
        "writer" => Some(writer()),
        _ => None,
    }
}

/// Catalog accessor — wraps the free-function API as a zero-sized struct.
pub struct AgentCatalog;

impl AgentCatalog {
    /// Returns all 10 pre-built agent manifests sorted alphabetically by name.
    pub fn all() -> Vec<AgentManifest> {
        catalog()
    }

    /// Returns the manifest for a specific agent by slug name, or `None` if unknown.
    pub fn get(name: &str) -> Option<AgentManifest> {
        get_agent(name)
    }
}
