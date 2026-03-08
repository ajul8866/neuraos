use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{chat, memory, task, agent, config};

#[derive(Parser)]
#[command(
    name = "neuraos",
    about = "NeuraOS - Intelligent Agent Operating System",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Increase verbosity
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format: text | json
    #[arg(long, global = true, default_value = "text")]
    pub output: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start an interactive chat session with an agent
    Chat(chat::ChatArgs),
    /// Manage memory entries
    Memory(memory::MemoryArgs),
    /// Manage tasks
    Task(task::TaskArgs),
    /// Manage agents
    Agent(agent::AgentArgs),
    /// Manage configuration
    Config(config::ConfigArgs),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Chat(args) => chat::run(args).await,
            Commands::Memory(args) => memory::run(args).await,
            Commands::Task(args) => task::run(args).await,
            Commands::Agent(args) => agent::run(args).await,
            Commands::Config(args) => config::run(args).await,
        }
    }
}
