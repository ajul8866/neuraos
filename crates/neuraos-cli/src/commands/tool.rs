//! Tool subcommand handlers.

use crate::client::NeuraClient;
use crate::output::*;
use crate::OutputFormat;
use colored::Colorize;
use serde_json::Value;
use tabled::Table;

use super::super::ToolExecArgs;

pub async fn cmd_tool_list(client: &NeuraClient, output: &OutputFormat) -> anyhow::Result<()> {
    let pb = spinner("Fetching tools...");
    let data = client.get("/v1/tools").await;
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
            let tools = val.as_array().cloned().unwrap_or_else(|| {
                val.get("tools")
                    .and_then(|a| a.as_array().cloned())
                    .unwrap_or_default()
            });
            if tools.is_empty() {
                print_warning("No tools registered.");
                return Ok(());
            }
            let rows: Vec<ToolRow> = tools
                .iter()
                .map(|t| ToolRow {
                    name: json_str(t, "name"),
                    category: json_str_or(t, "category", "general"),
                    description: truncate(&json_str_or(t, "description", "-"), 55),
                    enabled: if t.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true) {
                        "yes".green().to_string()
                    } else {
                        "no".red().to_string()
                    },
                })
                .collect();
            println!("{}", Table::new(rows));
            print_info("Total", &format!("{} tool(s)", tools.len()));
        }
    }
    Ok(())
}

pub async fn cmd_tool_exec(client: &NeuraClient, args: &ToolExecArgs) -> anyhow::Result<()> {
    let tool_args: Value = serde_json::from_str(&args.args)
        .map_err(|e| anyhow::anyhow!("Invalid JSON args: {e}"))?;

    let mut body = serde_json::json!({
        "tool": args.tool,
        "args": tool_args,
    });
    if let Some(agent) = &args.agent {
        body["agent_id"] = Value::String(agent.clone());
    }

    let pb = spinner(&format!("Executing tool '{}'...", args.tool));
    let data = client.post("/v1/tools/execute", &body).await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Tool execution failed: {e}"));
            std::process::exit(1);
        }
        Ok(val) => {
            print_success(&format!("Tool '{}' executed successfully", args.tool));
            println!("\n{}", "Result:".cyan().bold());
            println!("{}", serde_json::to_string_pretty(&val)?);
        }
    }
    Ok(())
}
