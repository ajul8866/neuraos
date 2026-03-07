//! Output formatting helpers, spinner, row structs, and print utilities for the NeuraOS CLI.

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::time::Duration;
use tabled::Tabled;

// ─── Table row types ──────────────────────────────────────────────────────────

#[derive(Debug, Tabled)]
pub struct AgentRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Role")]
    pub role: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Tasks")]
    pub tasks: String,
    #[tabled(rename = "Uptime")]
    pub uptime: String,
}

#[derive(Debug, Tabled)]
pub struct TaskRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Agent")]
    pub agent_id: String,
    #[tabled(rename = "Description")]
    pub description: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Priority")]
    pub priority: String,
    #[tabled(rename = "Created")]
    pub created_at: String,
}

#[derive(Debug, Tabled)]
pub struct ToolRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Category")]
    pub category: String,
    #[tabled(rename = "Description")]
    pub description: String,
    #[tabled(rename = "Enabled")]
    pub enabled: String,
}

#[derive(Debug, Tabled)]
pub struct MemoryRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Score")]
    pub score: String,
    #[tabled(rename = "Content")]
    pub content: String,
    #[tabled(rename = "Type")]
    pub memory_type: String,
}

// ─── Spinner ──────────────────────────────────────────────────────────────────

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

// ─── Print helpers ────────────────────────────────────────────────────────────

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

pub fn print_info(label: &str, value: &str) {
    println!("  {} {}", format!("{label}:").cyan().bold(), value);
}

// ─── Colorize ─────────────────────────────────────────────────────────────────

pub fn colorize_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "running" | "active" | "completed" | "success" => status.green().to_string(),
        "pending" | "queued" | "waiting" => status.yellow().to_string(),
        "failed" | "error" | "dead" | "killed" => status.red().to_string(),
        "idle" | "sleeping" => status.cyan().to_string(),
        _ => status.normal().to_string(),
    }
}

pub fn colorize_log_level(level: &str) -> String {
    match level.to_lowercase().as_str() {
        "error" => "ERROR".red().bold().to_string(),
        "warn" | "warning" => "WARN ".yellow().bold().to_string(),
        "info" => "INFO ".green().to_string(),
        "debug" => "DEBUG".blue().to_string(),
        "trace" => "TRACE".dimmed().to_string(),
        _ => level.normal().to_string(),
    }
}

// ─── JSON field extractors ────────────────────────────────────────────────────

pub fn json_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("-")
        .to_string()
}

pub fn json_str_or(v: &Value, key: &str, default: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or(default)
        .to_string()
}

pub fn json_u64(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_u64())
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string())
}

// ─── String utils ─────────────────────────────────────────────────────────────

pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
