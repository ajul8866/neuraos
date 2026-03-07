//! Memory subcommand handlers.

use crate::client::NeuraClient;
use crate::output::*;
use crate::OutputFormat;
use colored::Colorize;
use tabled::Table;

use super::super::{MemoryQueryArgs, MemoryClearArgs};

pub async fn cmd_memory_query(
    client: &NeuraClient,
    args: &MemoryQueryArgs,
    output: &OutputFormat,
) -> anyhow::Result<()> {
    let pb = spinner(&format!("Querying memory for agent {}...", args.agent));
    let limit = args.limit.to_string();
    let data = client
        .get_with_query(
            &format!("/v1/memory/{}", args.agent),
            &[("q", args.text.as_str()), ("limit", limit.as_str())],
        )
        .await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Memory query failed: {e}"));
            std::process::exit(1);
        }
        Ok(val) => {
            if matches!(output, OutputFormat::Json) {
                println!("{}", serde_json::to_string_pretty(&val)?);
                return Ok(());
            }
            let entries = val.as_array().cloned().unwrap_or_else(|| {
                val.get("entries")
                    .and_then(|a| a.as_array().cloned())
                    .unwrap_or_default()
            });
            if entries.is_empty() {
                print_warning("No memory entries found.");
                return Ok(());
            }
            let rows: Vec<MemoryRow> = entries
                .iter()
                .map(|e| MemoryRow {
                    id: json_str(e, "id"),
                    score: e.get("score")
                        .and_then(|s| s.as_f64())
                        .map(|f| format!("{:.3}", f))
                        .unwrap_or_else(|| "-".to_string()),
                    content: truncate(&json_str_or(e, "content", "-"), 60),
                    memory_type: json_str_or(e, "type", "episodic"),
                })
                .collect();
            println!("{}", Table::new(rows));
            print_info("Query", &args.text);
            print_info("Results", &format!("{} entry(ies)", entries.len()));
        }
    }
    Ok(())
}

pub async fn cmd_memory_clear(client: &NeuraClient, args: &MemoryClearArgs) -> anyhow::Result<()> {
    if !args.yes {
        print!("{} Clear ALL memory for agent {}? [y/N] ", "⚠".yellow().bold(), args.agent.bold());
        use std::io::{BufRead, Write};
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted.".dimmed());
            return Ok(());
        }
    }
    let pb = spinner(&format!("Clearing memory for agent {}...", args.agent));
    let data = client.delete(&format!("/v1/memory/{}", args.agent)).await;
    pb.finish_and_clear();

    match data {
        Err(e) => {
            print_error(&format!("Failed to clear memory: {e}"));
            std::process::exit(1);
        }
        Ok(_) => {
            print_success(&format!("Memory cleared for agent {}", args.agent));
        }
    }
    Ok(())
}
