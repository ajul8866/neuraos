//! Chat subcommand handler.

use crate::client::NeuraClient;
use crate::output::*;
use colored::Colorize;
use serde_json::Value;

use super::super::ChatArgs;

pub async fn cmd_chat(client: &NeuraClient, args: &ChatArgs) -> anyhow::Result<()> {
    let text = match &args.text {
        Some(t) => t.clone(),
        None => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf.trim().to_string()
        }
    };

    if text.is_empty() {
        print_error("No message text provided.");
        std::process::exit(1);
    }

    let mut body = serde_json::json!({
        "model": args.model.as_deref().unwrap_or("default"),
        "stream": args.stream,
        "messages": [
            {
                "role": "user",
                "content": text
            }
        ],
        "agent_id": args.agent,
    });

    if !args.stream {
        let pb = spinner("Waiting for response...");
        let data = client.post("/v1/chat/completions", &body).await;
        pb.finish_and_clear();

        match data {
            Err(e) => {
                print_error(&format!("Chat failed: {e}"));
                std::process::exit(1);
            }
            Ok(val) => {
                let content = val
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|c| c.get("message"))
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("[no response]");
                println!("\n{} {}", "Agent:".cyan().bold(), content);
            }
        }
    } else {
        // Streaming: use reqwest directly to consume SSE/chunked stream
        body["stream"] = Value::Bool(true);
        let mut req = client.client.post(client.url("/v1/chat/completions")).json(&body);
        if let Some(key) = &client.api_key {
            req = req.header("X-API-Key", key);
        }

        print!("\n{} ", "Agent:".cyan().bold());
        use std::io::Write;
        std::io::stdout().flush()?;

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await?;
            print_error(&format!("HTTP {} — {}", status, body_text));
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
                    if data_str == "[DONE]" {
                        break;
                    }
                    if let Ok(val) = serde_json::from_str::<Value>(data_str) {
                        if let Some(delta) = val
                            .get("choices")
                            .and_then(|c| c.as_array())
                            .and_then(|c| c.first())
                            .and_then(|c| c.get("delta"))
                            .and_then(|d| d.get("content"))
                            .and_then(|c| c.as_str())
                        {
                            print!("{delta}");
                            std::io::stdout().flush()?;
                        }
                    }
                }
            }
        }
        println!();
    }
    Ok(())
}
