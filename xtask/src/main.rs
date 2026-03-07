//! xtask -- build automation for NeuraOS.
//! Run with: `cargo xtask <command>`

use anyhow::{bail, Result};
use std::env;

fn main() -> Result<()> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("ci")     => ci(),
        Some("fmt")    => fmt(),
        Some("clippy") => clippy(),
        Some("test")   => test(),
        Some("build")  => build(),
        Some("docker") => docker(),
        Some(other)    => bail!(
            "Unknown task: {other}\nAvailable: ci, fmt, clippy, test, build, docker"
        ),
        None => {
            println!("Usage: cargo xtask <task>");
            println!("Tasks:");
            println!("  ci      -- run fmt + clippy + test");
            println!("  fmt     -- cargo fmt --all");
            println!("  clippy  -- cargo clippy --all-targets");
            println!("  test    -- cargo test --all");
            println!("  build   -- cargo build --release");
            println!("  docker  -- build Docker image");
            Ok(())
        }
    }
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = std::process::Command::new(cmd).args(args).status()?;
    if !status.success() {
        bail!("{cmd} failed with status: {status}");
    }
    Ok(())
}

fn ci() -> Result<()> {
    fmt()?;
    clippy()?;
    test()
}

fn fmt() -> Result<()> {
    run("cargo", &["fmt", "--all", "--", "--check"])
}

fn clippy() -> Result<()> {
    run("cargo", &["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"])
}

fn test() -> Result<()> {
    run("cargo", &["test", "--all"])
}

fn build() -> Result<()> {
    run("cargo", &["build", "--release"])
}

fn docker() -> Result<()> {
    run("docker", &["build", "-t", "neuraos:latest", "."])
}
