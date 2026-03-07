//! Agent subcommand handlers.

use crate::client::NeuraClient;
use crate::output::*;
use crate::OutputFormat;
use colored::Colorize;
use tabled::Table;

use super::super::{AgentSpawnArgs, AgentIdArgs};

pub async fn cmd_agent_list(client: &NeuraClient, output: &OutputFormat) -> anyhow::Result<()> {
    let pb = spinner("Fetching agents...");
    let data = client.get("/v1/agents").await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&e.to_string());
            std::process::exit(1);
        }
        Ok(val) => {
            if matches!(output, OutputFormat::Json) {
                println!("{}", serde_json::to_string_pretty(&val)?);
                return Ok(());
            }
            let agents = val.as_array().cloned().unwrap_or_else(|| {
                val.get("agents")
                    .and_then(|a| a.as_array().cloned())
                    .unwrap_or_default()
            });
            if agents.is_empty() {
                print_warning("No agents running.");
                return Ok(());
            }
            let rows: Vec<AgentRow> = agents
                .iter()
                .map(|a| AgentRow {
                    id: json_str(a, "id"),
                    name: json_str(a, "name"),
                    role: json_str_or(a, "role", "assistant"),
                    status: colorize_status(&json_str(a, "status")),
                    tasks: json_u64(a, "task_count"),
                    uptime: json_str_or(a, "uptime", "-"),
                })
                .collect();
            println!("{}", Table::new(rows));
            print_info("Total", &format!("{} agent(s)", agents.len()));
        }
    }
    Ok(())
}

pub async fn cmd_agent_spawn(client: &NeuraClient, args: &AgentSpawnArgs) -> anyhow::Result<()> {
    let pb = spinner(&format!("Spawning agent '{}'...", args.name));
    let mut body = serde_json::json!({
        "name": args.name,
        "role": args.role,
    });
    if let Some(task) = &args.task {
        body["task"] = serde_json::Value::String(task.clone());
    }
    if let Some(model) = &args.model {
        body["model"] = serde_json::Value::String(model.clone());
    }
    let data = client.post("/v1/agents", &body).await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Failed to spawn agent: {e}"));
            std::process::exit(1);
        }
        Ok(val) => {
            let id = json_str(&val, "id");
            print_success("Agent spawned successfully");
            print_info("ID", &id);
            print_info("Name", &args.name);
            print_info("Role", &args.role);
            if let Some(task) = &args.task {
                print_info("Task", task);
            }
        }
    }
    Ok(())
}

pub async fn cmd_agent_status(client: &NeuraClient, args: &AgentIdArgs) -> anyhow::Result<()> {
    let pb = spinner(&format!("Fetching agent {}...", args.id));
    let data = client.get(&format!("/v1/agents/{}", args.id)).await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Agent not found: {e}"));
            std::process::exit(1);
        }
        Ok(val) => {
            println!("{}", "Agent Status".bold().underline());
            print_info("ID", &json_str(&val, "id"));
            print_info("Name", &json_str(&val, "name"));
            print_info("Role", &json_str_or(&val, "role", "assistant"));
            print_info("Status", &colorize_status(&json_str(&val, "status")));
            print_info("Model", &json_str_or(&val, "model", "-"));
            print_info("Tasks completed", &json_u64(&val, "tasks_completed"));
            print_info("Tasks pending", &json_u64(&val, "tasks_pending"));
            print_info("Memory entries", &json_u64(&val, "memory_count"));
            print_info("Created", &json_str_or(&val, "created_at", "-"));
            print_info("Uptime", &json_str_or(&val, "uptime", "-"));
        }
    }
    Ok(())
}

pub async fn cmd_agent_kill(client: &NeuraClient, args: &AgentIdArgs) -> anyhow::Result<()> {
    let pb = spinner(&format!("Killing agent {}...", args.id));
    let data = client.delete(&format!("/v1/agents/{}", args.id)).await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Failed to kill agent: {e}"));
            std::process::exit(1);
        }
        Ok(_) => {
            print_success(&format!("Agent {} killed", args.id));
        }
    }
    Ok(())
}
