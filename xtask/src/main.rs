// xtask/src/main.rs
// NeuraOS build helper tasks (xtask pattern)

use std::process::Command;

fn main() {
    let task = std::env::args().nth(1).unwrap_or_else(|| "help".to_string());
    match task.as_str() {
        "build"       => build(),
        "test"        => test(),
        "lint"        => lint(),
        "fmt"         => fmt(),
        "doc"         => doc(),
        "clean"       => clean(),
        "ci"          => { fmt(); lint(); test(); build(); }
        _             => help(),
    }
}

fn build() {
    run("cargo", &["build", "--workspace", "--all-features"]);
}

fn test() {
    run("cargo", &["test", "--workspace", "--all-features"]);
}

fn lint() {
    run("cargo", &["clippy", "--workspace", "--all-features", "--", "-D", "warnings"]);
}

fn fmt() {
    run("cargo", &["fmt", "--all"]);
}

fn doc() {
    run("cargo", &["doc", "--workspace", "--no-deps", "--open"]);
}

fn clean() {
    run("cargo", &["clean"]);
}

fn help() {
    println!("Available xtask commands:");
    println!("  build   -- cargo build --workspace");
    println!("  test    -- cargo test --workspace");
    println!("  lint    -- cargo clippy");
    println!("  fmt     -- cargo fmt --all");
    println!("  doc     -- cargo doc --workspace");
    println!("  clean   -- cargo clean");
    println!("  ci      -- fmt + lint + test + build");
}

fn run(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd).args(args).status().expect("failed to run command");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
