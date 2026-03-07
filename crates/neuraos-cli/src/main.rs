//! neura — NeuraOS command-line interface entry point.
//!
//! This file contains only CLI structure definitions and the main dispatcher.
//! Business logic lives in the submodules:
//!   - [`client`]   — HTTP client for the NeuraOS API
//!   - [`output`]   — formatting, spinners, print helpers
//!   - [`commands`] — per-subcommand handlers

mod client;
mod output;
mod commands;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use serde_json::Value;
use tracing::debug;

use client::NeuraClient;
use output::{print_error, print_info, print_success, print_warning, colorize_log_level, json_str_or, json_u64, spinner};

// ─── Top-level CLI ────────────────────────────────────────────────────────────

/// neura — NeuraOS command-line interface
#[derive(Debug, Parser)]
#[command(
    name = "neura",
    author,
    version,
    about = "NeuraOS CLI — spawn agents, manage tasks, query memory, run tools",
    long_about = None,
    propagate_version = true,
)]
struct Cli {
    /// NeuraOS server URL
    #[arg(long, env = "NEURAOS_URL", default_value = "http://localhost:8080", global = true)]
    server: String,

    /// API key for authentication
    #[arg(long, env = "NEURAOS_API_KEY", global = true)]
    api_key: Option<String>,

    /// Output format: table | json | plain
    #[arg(long, default_value = "table", global = true)]
    output: OutputFormat,

    /// Enable verbose/debug logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Plain,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Plain => write!(f, "plain"),
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Manage agents
    #[command(subcommand)]
    Agent(AgentCommands),

    /// Manage tasks
    #[command(subcommand)]
    Task(TaskCommands),

    /// Query and manage memory
    #[command(subcommand)]
    Memory(MemoryCommands),

    /// Manage tools
    #[command(subcommand)]
    Tool(ToolCommands),

    /// Chat with an agent
    Chat(ChatArgs),

    /// Show current configuration
    Config(ConfigArgs),

    /// Manage the NeuraOS server process
    #[command(subcommand)]
    Server(ServerCommands),

    /// Stream server logs (SSE)
    Logs(LogsArgs),
}

// ─── Agent subcommands ────────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
enum AgentCommands {
    /// List all running agents
    List,

    /// Spawn a new agent
    Spawn(AgentSpawnArgs),

    /// Show agent status
    Status(AgentIdArgs),

    /// Kill an agent
    Kill(AgentIdArgs),
}

#[derive(Debug, Args)]
pub struct AgentSpawnArgs {
    /// Agent name
    pub name: String,

    /// Initial task description
    #[arg(long)]
    pub task: Option<String>,

    /// Agent role/type
    #[arg(long, default_value = "assistant")]
    pub role: String,

    /// Agent model override
    #[arg(long)]
    pub model: Option<String>,
}

#[derive(Debug, Args)]
pub struct AgentIdArgs {
    /// Agent ID
    pub id: String,
}

// ─── Task subcommands ─────────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
enum TaskCommands {
    /// List all tasks
    List,

    /// Show task status
    Status(TaskIdArgs),

    /// Cancel a task
    Cancel(TaskIdArgs),
}

#[derive(Debug, Args)]
pub struct TaskIdArgs {
    /// Task ID
    pub id: String,
}

// ─── Memory subcommands ───────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
enum MemoryCommands {
    /// Query agent memory
    Query(MemoryQueryArgs),

    /// Clear agent memory
    Clear(MemoryClearArgs),
}

#[derive(Debug, Args)]
pub struct MemoryQueryArgs {
    /// Agent ID to query memory for
    #[arg(long)]
    pub agent: String,

    /// Search text
    #[arg(long)]
    pub text: String,

    /// Maximum results to return
    #[arg(long, default_value = "10")]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct MemoryClearArgs {
    /// Agent ID to clear memory for
    #[arg(long)]
    pub agent: String,

    /// Skip confirmation prompt
    #[arg(long)]
    pub yes: bool,
}

// ─── Tool subcommands ─────────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
enum ToolCommands {
    /// List all registered tools
    List,

    /// Execute a tool
    Exec(ToolExecArgs),
}

#[derive(Debug, Args)]
pub struct ToolExecArgs {
    /// Tool name to execute
    #[arg(long)]
    pub tool: String,

    /// Tool arguments as JSON string
    #[arg(long, default_value = "{}")]
    pub args: String,

    /// Agent context ID (optional)
    #[arg(long)]
    pub agent: Option<String>,
}

// ─── Chat ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct ChatArgs {
    /// Agent ID to chat with
    #[arg(long)]
    pub agent: String,

    /// Message text (reads from stdin if omitted)
    pub text: Option<String>,

    /// Stream the response token-by-token
    #[arg(long, default_value = "true")]
    pub stream: bool,

    /// Model to use
    #[arg(long)]
    pub model: Option<String>,
}

// ─── Config ───────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
struct ConfigArgs {
    /// Show config as raw JSON
    #[arg(long)]
    json: bool,
}

// ─── Server subcommands ───────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
enum ServerCommands {
    /// Start the NeuraOS server
    Start(ServerStartArgs),

    /// Show server health/status
    Status,
}

#[derive(Debug, Args)]
struct ServerStartArgs {
    /// Port to listen on
    #[arg(long, short, default_value = "8080")]
    port: u16,

    /// Config file path
    #[arg(long, default_value = "config/default.toml")]
    config: String,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Number of worker threads
    #[arg(long)]
    workers: Option<usize>,
}

// ─── Logs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct LogsArgs {
    /// Follow log stream (SSE)
    #[arg(long, short)]
    pub follow: bool,

    /// Filter by log level: trace|debug|info|warn|error
    #[arg(long, default_value = "info")]
    pub level: String,

    /// Filter by agent ID
    #[arg(long)]
    pub agent: Option<String>,

    /// Max lines to show (non-follow mode)
    #[arg(long, default_value = "100")]
    pub lines: usize,
}

// ─── Inline handlers (config, server, logs) ───────────────────────────────────
// These are short enough to stay in main.rs; they don't depend on HTTP responses
// in complex ways.

fn cmd_config_show(cli: &Cli, args: &ConfigArgs) -> anyhow::Result<()> {
    let config = serde_json::json!({
        "server": cli.server,
        "api_key": cli.api_key.as_deref().map(|k| {
            if k.len() > 8 { format!("{}...", &k[..8]) } else { "***".to_string() }
        }),
        "output_format": cli.output.to_string(),
        "verbose": cli.verbose,
    });

    if args.json {
        println!("{}", serde_json::to_string_pretty(&config)?);
    } else {
        println!("{}", "NeuraOS CLI Configuration".bold().underline());
        print_info("Server URL", &cli.server);
        print_info(
            "API Key",
            &cli.api_key
                .as_deref()
                .map(|k| if k.len() > 8 { format!("{}...", &k[..8]) } else { "***".to_string() })
                .unwrap_or_else(|| "(not set)".to_string()),
        );
        print_info("Output Format", &cli.output.to_string());
        print_info("Verbose", if cli.verbose { "yes" } else { "no" });
        println!();
        print_info("Env: NEURAOS_URL", "(server URL override)");
        print_info("Env: NEURAOS_API_KEY", "(API key override)");
    }
    Ok(())
}

fn cmd_server_start(args: &ServerStartArgs) -> anyhow::Result<()> {
    use std::process::Command;
    let workers = args.workers.unwrap_or_else(num_cpus);
    println!("{}", "Starting NeuraOS server...".bold());
    print_info("Port", &args.port.to_string());
    print_info("Config", &args.config);
    print_info("Log level", &args.log_level);
    print_info("Workers", &workers.to_string());

    let mut cmd = Command::new("neuraos");
    cmd.arg("--port").arg(args.port.to_string())
        .arg("--config").arg(&args.config)
        .arg("--log-level").arg(&args.log_level)
        .arg("--workers").arg(workers.to_string());

    match cmd.spawn() {
        Ok(child) => {
            print_success(&format!("NeuraOS server started (PID {})", child.id()));
        }
        Err(e) => {
            print_error(&format!("Failed to start server: {e}"));
            print_warning("Make sure 'neuraos' binary is in your PATH.");
            std::process::exit(1);
        }
    }
    Ok(())
}

async fn cmd_server_status(client: &NeuraClient) -> anyhow::Result<()> {
    let pb = spinner("Checking server health...");
    let data = client.get("/health").await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Server unreachable: {e}"));
            print_info("URL", &client.base_url);
            std::process::exit(1);
        }
        Ok(val) => {
            print_success("Server is healthy");
            print_info("Status", &json_str_or(&val, "status", "ok"));
            print_info("Version", &json_str_or(&val, "version", "-"));
            print_info("Uptime", &json_str_or(&val, "uptime", "-"));
            print_info("Agents", &json_u64(&val, "agent_count"));
            print_info("Tasks", &json_u64(&val, "task_count"));
        }
    }
    Ok(())
}

async fn cmd_logs(client: &NeuraClient, args: &LogsArgs) -> anyhow::Result<()> {
    if args.follow {
        println!("{}", format!("Streaming logs (level={}, Ctrl-C to stop)...", args.level).dimmed());
        let level = args.level.as_str();
        let mut params = vec![("level", level)];
        let agent_str;
        if let Some(agent) = &args.agent {
            agent_str = agent.clone();
            params.push(("agent_id", agent_str.as_str()));
        }

        let mut req = client.client.get(client.url("/v1/events")).query(&params);
        if let Some(key) = &client.api_key {
            req = req.header("X-API-Key", key);
        }
        req = req.header("Accept", "text/event-stream");

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await?;
            print_error(&format!("HTTP {} — {}", status, body));
            std::process::exit(1);
        }

        let mut stream = resp.bytes_stream();
        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with("data: ") {
                    let data_str = &line["data: ".len()..];
                    if let Ok(val) = serde_json::from_str::<Value>(data_str) {
                        let level = json_str_or(&val, "level", "info");
                        let ts = json_str_or(&val, "timestamp", "-");
                        let msg = json_str_or(&val, "message", "-");
                        let agent = json_str_or(&val, "agent_id", "");
                        let level_colored = colorize_log_level(&level);
                        if agent.is_empty() {
                            println!("{} [{}] {}", ts.dimmed(), level_colored, msg);
                        } else {
                            println!("{} [{}] [{}] {}", ts.dimmed(), level_colored, agent.cyan(), msg);
                        }
                    } else {
                        println!("{}", data_str);
                    }
                }
            }
        }
    } else {
        let level = args.level.as_str();
        let lines_str = args.lines.to_string();
        let mut params = vec![("level", level), ("limit", lines_str.as_str())];
        let agent_str;
        if let Some(agent) = &args.agent {
            agent_str = agent.clone();
            params.push(("agent_id", agent_str.as_str()));
        }

        let pb = spinner("Fetching logs...");
        let data = client.get_with_query("/v1/events", &params).await;
        pb.finish_and_clear();

        match data {
            Err(e) => {
                print_error(&e.to_string());
                std::process::exit(1);
            }
            Ok(val) => {
                let entries = val.as_array().cloned().unwrap_or_else(|| {
                    val.get("events")
                        .and_then(|a| a.as_array().cloned())
                        .unwrap_or_default()
                });
                for entry in &entries {
                    let level = json_str_or(entry, "level", "info");
                    let ts = json_str_or(entry, "timestamp", "-");
                    let msg = json_str_or(entry, "message", "-");
                    let agent = json_str_or(entry, "agent_id", "");
                    let level_colored = colorize_log_level(&level);
                    if agent.is_empty() {
                        println!("{} [{}] {}", ts.dimmed(), level_colored, msg);
                    } else {
                        println!("{} [{}] [{}] {}", ts.dimmed(), level_colored, agent.cyan(), msg);
                    }
                }
                print_info("Lines", &format!("{}", entries.len()));
            }
        }
    }
    Ok(())
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let level = if cli.verbose { "debug" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
        )
        .with_target(false)
        .compact()
        .init();

    debug!("CLI args parsed: server={}", cli.server);

    let client = NeuraClient::new(cli.server.clone(), cli.api_key.clone())?;

    match &cli.command {
        // ── Agent ───────────────────────────────────────────────────────────
        Commands::Agent(AgentCommands::List) => {
            commands::agent::cmd_agent_list(&client, &cli.output).await?;
        }
        Commands::Agent(AgentCommands::Spawn(args)) => {
            commands::agent::cmd_agent_spawn(&client, args).await?;
        }
        Commands::Agent(AgentCommands::Status(args)) => {
            commands::agent::cmd_agent_status(&client, args).await?;
        }
        Commands::Agent(AgentCommands::Kill(args)) => {
            commands::agent::cmd_agent_kill(&client, args).await?;
        }

        // ── Task ────────────────────────────────────────────────────────────
        Commands::Task(TaskCommands::List) => {
            commands::task::cmd_task_list(&client, &cli.output).await?;
        }
        Commands::Task(TaskCommands::Status(args)) => {
            commands::task::cmd_task_status(&client, args).await?;
        }
        Commands::Task(TaskCommands::Cancel(args)) => {
            commands::task::cmd_task_cancel(&client, args).await?;
        }

        // ── Memory ──────────────────────────────────────────────────────────
        Commands::Memory(MemoryCommands::Query(args)) => {
            commands::memory::cmd_memory_query(&client, args, &cli.output).await?;
        }
        Commands::Memory(MemoryCommands::Clear(args)) => {
            commands::memory::cmd_memory_clear(&client, args).await?;
        }

        // ── Tool ────────────────────────────────────────────────────────────
        Commands::Tool(ToolCommands::List) => {
            commands::tool::cmd_tool_list(&client, &cli.output).await?;
        }
        Commands::Tool(ToolCommands::Exec(args)) => {
            commands::tool::cmd_tool_exec(&client, args).await?;
        }

        // ── Chat ────────────────────────────────────────────────────────────
        Commands::Chat(args) => {
            commands::chat::cmd_chat(&client, args).await?;
        }

        // ── Config ──────────────────────────────────────────────────────────
        Commands::Config(args) => {
            cmd_config_show(&cli, args)?;
        }

        // ── Server ──────────────────────────────────────────────────────────
        Commands::Server(ServerCommands::Start(args)) => {
            cmd_server_start(args)?;
        }
        Commands::Server(ServerCommands::Status) => {
            cmd_server_status(&client).await?;
        }

        // ── Logs ────────────────────────────────────────────────────────────
        Commands::Logs(args) => {
            cmd_logs(&client, args).await?;
        }
    }

    Ok(())
}
