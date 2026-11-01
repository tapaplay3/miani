mod client;
mod config;
mod modes;
mod state;

use anyhow::Result;
use clap::Parser;
use colored::*;
use client::{LLMClient, Message};
use config::Provider;
use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Minimalist CLI Agent for Local LLMs
#[derive(Parser, Debug)]
#[command(name = "miani")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Provider to use (lmstudio or ollama)
    #[arg(short = 'p', long, default_value = "lmstudio")]
    provider: String,

    /// Model name to use
    #[arg(short, long)]
    model: Option<String>,

    /// Continue from saved conversation
    #[arg(long)]
    r#continue: bool,

    /// API endpoint (overrides provider default)
    #[arg(short, long)]
    endpoint: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse provider
    let provider = Provider::from_str(&args.provider).unwrap_or_else(|e| {
        eprintln!("{} Using LM Studio as fallback.", e.yellow());
        Provider::LMStudio
    });

    // Determine endpoint (CLI arg > provider default)
    let endpoint = args.endpoint.unwrap_or_else(|| provider.default_endpoint().to_string());

    // Determine model
    let config = config::Config::new();
    let model = args.model.unwrap_or(config.model);

    let client = LLMClient::new(provider, endpoint.clone(), model.clone());

    // Setup Ctrl+C handler
    let interrupt_flag = Arc::new(AtomicBool::new(false));
    let interrupt_flag_clone = interrupt_flag.clone();
    let last_interrupt_time = Arc::new(AtomicU64::new(0));
    let last_interrupt_time_clone = last_interrupt_time.clone();

    ctrlc::set_handler(move || {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last = last_interrupt_time_clone.load(Ordering::SeqCst);

        // Double Ctrl+C within 2 seconds = exit
        if now - last < 2 {
            println!("\n{}", "Exiting...".yellow());
            std::process::exit(0);
        }

        // Single Ctrl+C = interrupt current response silently
        last_interrupt_time_clone.store(now, Ordering::SeqCst);
        interrupt_flag_clone.store(true, Ordering::SeqCst);
    })?;

    // Display ASCII art banner
    println!();
    println!("{}", r"  ███╗   ███╗██╗ █████╗ ███╗   ██╗██╗".cyan().bold());
    println!("{}", r"  ████╗ ████║██║██╔══██╗████╗  ██║██║".cyan().bold());
    println!("{}", r"  ██╔████╔██║██║███████║██╔██╗ ██║██║".cyan().bold());
    println!("{}", r"  ██║╚██╔╝██║██║██╔══██║██║╚██╗██║██║".cyan().bold());
    println!("{}", r"  ██║ ╚═╝ ██║██║██║  ██║██║ ╚████║██║".cyan().bold());
    println!("{}", r"  ╚═╝     ╚═╝╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝".cyan().bold());
    println!();
    println!("{}", "  Minimalist CLI Agent for Local LLMs".dimmed());
    println!();

    // Check provider connection
    println!("{}", format!("Checking connection to {}...", client.get_provider_name()).cyan());
    println!("{}", format!("  Endpoint: {}", endpoint).dimmed());
    if !client.check_connection().await? {
        eprintln!(
            "{} {}",
            "✗".red().bold(),
            format!("Cannot connect to {}. {}", client.get_provider_name(), client.get_setup_hint()).red()
        );
        std::process::exit(1);
    }

    println!("{}", format!("✓ Connected to {}", client.get_provider_name()).green());

    // Get and display loaded model
    match client.get_loaded_model().await {
        Ok(model_name) => {
            println!("{} {}", "Loaded model:".cyan(), model_name.yellow().bold());
        }
        Err(_) => {
            println!("{} {}", "Model:".cyan(), "unknown".yellow());
        }
    }

    println!();
    println!("{}", "CLI Agent ready.".cyan());
    println!("{}", "  Type your message to chat".dimmed());
    println!("{}", "  Type ! for bash commands (e.g., !ls -la)".dimmed());
    println!("{}", "  Type @ for tool modes (e.g., @file, @git, @grep)".dimmed());
    println!("{}", "  Type /help for all commands".dimmed());
    println!();

    // Load or create conversation state
    let state_path = state::ConversationState::get_state_path()?;
    let mut conversation: Vec<Message> = if args.r#continue && state_path.exists() {
        match state::ConversationState::load(&state_path) {
            Ok(saved_state) => {
                println!("{}", "📝 Restored previous conversation".green());
                println!("{} {} messages", "   ".dimmed(), saved_state.messages.len().to_string().yellow());

                // Warn if provider changed
                if let Some(saved_provider) = &saved_state.provider {
                    if saved_provider != &provider.to_string() {
                        println!("{} Provider changed from {} to {}",
                            "⚠".yellow(),
                            saved_provider.yellow(),
                            provider.to_string().yellow());
                    }
                }
                println!();
                saved_state.messages
            }
            Err(e) => {
                eprintln!("{} Failed to load conversation: {}", "⚠".yellow(), e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    loop {
        // Display prompt
        print!("{} ", ">".green().bold());
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Handle commands
        if input.is_empty() {
            continue;
        }

        // Bash command mode
        if input.starts_with('!') {
            let bash_cmd = &input[1..].trim();
            execute_bash_command(bash_cmd)?;
            continue;
        }

        // Tool modes (like Claude Code)
        if input.starts_with('@') {
            let rest = &input[1..];
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            let mode = parts[0];
            let args = parts.get(1).unwrap_or(&"");

            match mode {
                "file" => modes::handle_file_command(args)?,
                "git" => modes::handle_git_command(args)?,
                "grep" => modes::handle_grep_command(args)?,
                "tree" => modes::handle_tree_command(args)?,
                "code" => modes::handle_code_command(args)?,
                _ => {
                    println!("{} Unknown mode: @{}", "✗".red().bold(), mode);
                    println!("{}", "Available modes: @file @git @grep @tree @code".yellow());
                }
            }
            continue;
        }

        if input == "/exit" || input == "/quit" {
            // Save conversation state before exit
            let save_state = state::ConversationState {
                messages: conversation.clone(),
                model: model.clone(),
                provider: Some(provider.to_string()),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            match save_state.save(&state_path) {
                Ok(_) => println!("{}", "💾 Conversation saved".green()),
                Err(e) => eprintln!("{} Failed to save: {}", "⚠".yellow(), e),
            }

            println!("{}", "Goodbye! 👋".cyan());
            break;
        }

        if input == "/clear" {
            conversation.clear();
            println!("{}", "Conversation cleared.".yellow());
            continue;
        }

        if input == "/model" {
            match client.get_loaded_model().await {
                Ok(model_name) => {
                    println!("{} {}", "Current model:".cyan(), model_name.yellow().bold());
                }
                Err(e) => {
                    eprintln!("{} {}", "✗".red().bold(), format!("Error: {}", e).red());
                }
            }
            continue;
        }

        if input == "/help" {
            println!("{}", "Available commands:".cyan().bold());
            println!();
            println!("{}", "Chat Mode:".cyan());
            println!("  {} <message>        - Chat with AI", ">".green());
            println!();
            println!("{}", "Bash Mode:".cyan());
            println!("  {} <command>        - Execute bash command", "!".yellow());
            println!("  {} ls -la            Example: list files", "!".dimmed());
            println!("  {} git status        Example: check git status", "!".dimmed());
            println!();
            println!("{}", "Tool Modes (Claude Code style):".cyan());
            println!("  {} <path>         - Read and display file", "@file".yellow());
            println!("  {} <cmd>          - Git operations (status, diff, log)", "@git".yellow());
            println!("  {} <pattern>      - Search for pattern in files", "@grep".yellow());
            println!("  {} [path]         - Show directory tree", "@tree".yellow());
            println!("  {} <cmd>          - Code tools (review, format, lint)", "@code".yellow());
            println!();
            println!("{}", "Examples:".dimmed());
            println!("  @file src/main.rs");
            println!("  @git status");
            println!("  @git diff");
            println!("  @git log 5");
            println!("  @grep TODO .");
            println!("  @tree src/");
            println!("  @code format");
            println!("  @code lint");
            println!();
            println!("{}", "Special Commands:".cyan());
            println!("  {}              - Show loaded model info", "/model".yellow());
            println!("  {}              - Clear conversation history", "/clear".yellow());
            println!("  {}               - Show this help message", "/help".yellow());
            println!("  {}               - Exit the application", "/exit".yellow());
            println!();
            continue;
        }

        // Add user message to conversation
        conversation.push(Message {
            role: "user".to_string(),
            content: input.to_string(),
        });

        // Reset interrupt flag before streaming
        interrupt_flag.store(false, Ordering::SeqCst);

        // Stream response from LM Studio
        print!("{} ", "AI:".blue().bold());
        io::stdout().flush()?;

        let interrupt_check = interrupt_flag.clone();
        match client
            .chat_stream(conversation.clone(), |chunk| {
                // Check if interrupted
                if interrupt_check.load(Ordering::SeqCst) {
                    return;
                }
                print!("{}", chunk);
                io::stdout().flush().ok();
            })
            .await
        {
            Ok(response) => {
                if interrupt_flag.load(Ordering::SeqCst) {
                    println!();
                    println!("{}", "[Response interrupted]".yellow());
                    println!();
                    // Don't add interrupted response to conversation
                    conversation.pop(); // Remove the user message we just added
                } else {
                    println!();
                    println!();

                    // Add assistant response to conversation
                    conversation.push(Message {
                        role: "assistant".to_string(),
                        content: response,
                    });
                }
            }
            Err(e) => {
                println!();
                eprintln!("{} {}", "✗".red().bold(), format!("Error: {}", e).red());
                println!();
                conversation.pop(); // Remove the user message on error
            }
        }
    }

    Ok(())
}

fn execute_bash_command(cmd: &str) -> Result<()> {
    if cmd.is_empty() {
        println!("{}", "Empty command".yellow());
        return Ok(());
    }

    println!("{} {}", "Executing:".dimmed(), cmd.cyan());
    println!();

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
    };

    match output {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr).red());
            }
            if !output.status.success() {
                println!(
                    "{}",
                    format!("Command exited with status: {}", output.status).red()
                );
            }
        }
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), format!("Failed to execute: {}", e).red());
        }
    }

    println!();
    Ok(())
}
