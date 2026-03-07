//! Task subcommand handlers.

use crate::client::NeuraClient;
use crate::output::*;
use crate::OutputFormat;
use colored::Colorize;
use tabled::Table;

use super::super::TaskIdArgs;

pub async fn cmd_task_list(client: &NeuraClient, output: &OutputFormat) -> anyhow::Result<()> {
    let pb = spinner("Fetching tasks...");
    let data = client.get("/v1/tasks").await;
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
            let tasks = val.as_array().cloned().unwrap_or_else(|| {
                val.get("tasks")
                    .and_then(|a| a.as_array().cloned())
                    .unwrap_or_default()
            });
            if tasks.is_empty() {
                print_warning("No tasks found.");
                return Ok(());
            }
            let rows: Vec<TaskRow> = tasks
                .iter()
                .map(|t| TaskRow {
                    id: json_str(t, "id"),
                    agent_id: json_str_or(t, "agent_id", "-"),
                    description: truncate(&json_str_or(t, "description", "-"), 50),
                    status: colorize_status(&json_str(t, "status")),
                    priority: json_str_or(t, "priority", "normal"),
                    created_at: json_str_or(t, "created_at", "-"),
                })
                .collect();
            println!("{}", Table::new(rows));
            print_info("Total", &format!("{} task(s)", tasks.len()));
        }
    }
    Ok(())
}

pub async fn cmd_task_status(client: &NeuraClient, args: &TaskIdArgs) -> anyhow::Result<()> {
    let pb = spinner(&format!("Fetching task {}...", args.id));
    let data = client.get(&format!("/v1/tasks/{}", args.id)).await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Task not found: {e}"));
            std::process::exit(1);
        }
        Ok(val) => {
            println!("{}", "Task Status".bold().underline());
            print_info("ID", &json_str(&val, "id"));
            print_info("Agent", &json_str_or(&val, "agent_id", "-"));
            print_info("Description", &json_str_or(&val, "description", "-"));
            print_info("Status", &colorize_status(&json_str(&val, "status")));
            print_info("Priority", &json_str_or(&val, "priority", "normal"));
            print_info("Created", &json_str_or(&val, "created_at", "-"));
            print_info("Updated", &json_str_or(&val, "updated_at", "-"));
            if let Some(result) = val.get("result") {
                println!("\n  {} {}", "Result:".cyan().bold(), serde_json::to_string_pretty(result)?);
            }
            if let Some(error) = val.get("error").and_then(|e| e.as_str()) {
                println!("\n  {} {}", "Error:".red().bold(), error);
            }
        }
    }
    Ok(())
}

pub async fn cmd_task_cancel(client: &NeuraClient, args: &TaskIdArgs) -> anyhow::Result<()> {
    let pb = spinner(&format!("Cancelling task {}...", args.id));
    let data = client.delete(&format!("/v1/tasks/{}", args.id)).await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Failed to cancel task: {e}"));
            std::process::exit(1);
        }
        Ok(_) => {
            print_success(&format!("Task {} cancelled", args.id));
        }
    }
    Ok(())
}
