use anyhow::Result;
use colored::*;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn handle_file_command(args: &str) -> Result<()> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.is_empty() {
        println!("{}", "Usage: @file <path>".yellow());
        return Ok(());
    }

    let file_path = parts[0];

    if !Path::new(file_path).exists() {
        println!("{} File not found: {}", "✗".red().bold(), file_path);
        return Ok(());
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            println!("{} {}", "File:".cyan().bold(), file_path.yellow());
            println!("{}", "─".repeat(60).dimmed());
            println!("{}", content);
            println!("{}", "─".repeat(60).dimmed());
        }
        Err(e) => {
            println!("{} Failed to read file: {}", "✗".red().bold(), e);
        }
    }

    Ok(())
}

pub fn handle_git_command(args: &str) -> Result<()> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.is_empty() {
        println!("{}", "Usage: @git <status|diff|log>".yellow());
        return Ok(());
    }

    let subcommand = parts[0];

    match subcommand {
        "status" => {
            execute_git(&["status", "--short"])
        }
        "diff" => {
            execute_git(&["diff", "--color=always"])
        }
        "log" => {
            let count = parts.get(1).unwrap_or(&"10");
            execute_git(&["log", "--oneline", &format!("-{}", count)])
        }
        _ => {
            println!("{}", "Available: status, diff, log".yellow());
            Ok(())
        }
    }
}

pub fn handle_grep_command(args: &str) -> Result<()> {
    let parts: Vec<&str> = args.splitn(2, ' ').collect();

    if parts.is_empty() || parts[0].is_empty() {
        println!("{}", "Usage: @grep <pattern> [path]".yellow());
        return Ok(());
    }

    let pattern = parts[0];
    let path = parts.get(1).unwrap_or(&".");

    let output = if cfg!(target_os = "windows") {
        Command::new("findstr")
            .args(["/s", "/n", pattern, path])
            .output()
    } else {
        Command::new("grep")
            .args(["-rn", "--color=always", pattern, path])
            .output()
    };

    match output {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                println!("{}", "No matches found".yellow());
            }
        }
        Err(e) => {
            println!("{} Failed to execute grep: {}", "✗".red().bold(), e);
        }
    }

    Ok(())
}

pub fn handle_tree_command(args: &str) -> Result<()> {
    let path = if args.is_empty() { "." } else { args.trim() };

    let output = if cfg!(target_os = "windows") {
        Command::new("tree")
            .arg("/F")
            .arg(path)
            .output()
    } else {
        Command::new("tree")
            .args(["-L", "3", path])
            .output()
    };

    match output {
        Ok(output) => {
            if output.status.success() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                // Fallback если tree не установлен
                println!("{}", "Tree command not available. Using ls:".yellow());
                execute_ls(path)?;
            }
        }
        Err(_) => {
            println!("{}", "Tree command not available. Using ls:".yellow());
            execute_ls(path)?;
        }
    }

    Ok(())
}

pub fn handle_code_command(args: &str) -> Result<()> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.is_empty() {
        println!("{}", "Usage: @code <review|format|lint> [file]".yellow());
        return Ok(());
    }

    let subcommand = parts[0];

    match subcommand {
        "review" => {
            let file = parts.get(1).unwrap_or(&"src/");
            println!("{} {}", "Reviewing:".cyan().bold(), file.yellow());
            println!("{}", "This would analyze code for issues...".dimmed());
            Ok(())
        }
        "format" => {
            println!("{}", "Formatting Rust code...".cyan());
            execute_cargo(&["fmt"])
        }
        "lint" => {
            println!("{}", "Running clippy...".cyan());
            execute_cargo(&["clippy", "--", "-W", "clippy::pedantic"])
        }
        _ => {
            println!("{}", "Available: review, format, lint".yellow());
            Ok(())
        }
    }
}

fn execute_git(args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

fn execute_cargo(args: &[&str]) -> Result<()> {
    let output = Command::new("cargo")
        .args(args)
        .output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

fn execute_ls(path: &str) -> Result<()> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "dir", path])
            .output()
    } else {
        Command::new("ls")
            .args(["-la", path])
            .output()
    };

    match output {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
        Err(e) => {
            println!("{} Failed to list directory: {}", "✗".red().bold(), e);
        }
    }

    Ok(())
}
