use clap::{Parser, Subcommand};
use anyhow::Result;

mod publish;
mod install;
mod search;
mod github;
mod extract;
mod config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "claude-md-snippets")]
#[command(about = "Manage and share CLAUDE.md snippets")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish a snippet to the repository
    Publish {
        /// The snippet content to publish (if not using --file)
        content: Option<String>,
        /// Custom name for the snippet (optional)
        #[arg(short, long)]
        name: Option<String>,
        /// Publish from a saved snippet file
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Install a snippet to local CLAUDE.md
    Install {
        /// Description to find the relevant snippet
        query: String,
    },
    /// Search snippets with fuzzy finder
    Search,
    /// Sync snippets with GitHub repository
    Sync,
    /// Pull latest snippets from repository
    Pull,
    /// Extract relevant information from ~/.claude/CLAUDE.md
    Extract {
        /// Topic or query to extract information about
        query: String,
    },
    /// Setup GitHub repository for snippets
    Setup {
        /// Repository name (defaults to 'default')
        #[arg(short, long)]
        repo: Option<String>,
    },
    /// Show status of repositories and current default
    Status,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        config_command: ConfigCommand,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Set the default repository
    SetDefault {
        /// Repository name
        repo_name: String,
    },
    /// Show current configuration
    Show,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Publish { content, name, file } => {
            publish::publish_snippet(content, name, file).await?;
        }
        Commands::Install { query } => {
            install::install_snippet(query).await?;
        }
        Commands::Search => {
            search::search_snippets().await?;
        }
        Commands::Sync => {
            github::sync_snippets().await?;
        }
        Commands::Pull => {
            github::pull_snippets().await?;
        }
        Commands::Extract { query } => {
            extract::extract_snippet(query).await?;
        }
        Commands::Setup { repo } => {
            github::setup_repository(repo).await?;
        }
        Commands::Status => {
            show_status().await?;
        }
        Commands::Config { config_command } => {
            match config_command {
                ConfigCommand::SetDefault { repo_name } => {
                    set_default_repo(repo_name).await?;
                }
                ConfigCommand::Show => {
                    show_config().await?;
                }
            }
        }
    }

    Ok(())
}

async fn show_status() -> Result<()> {
    use std::fs;
    use publish::{get_repos_dir, get_default_repo_dir};
    
    println!("📊 Claude MD Snippets Status");
    println!("============================");
    
    let repos_dir = get_repos_dir()?;
    
    if !repos_dir.exists() {
        println!("❌ No repositories directory found at: {}", repos_dir.display());
        println!("💡 Run 'claude-md-snippets setup' to create your first repository");
        return Ok(());
    }
    
    // List all repositories
    println!("📁 Repositories:");
    let mut repos = Vec::new();
    
    for entry in fs::read_dir(&repos_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                repos.push(name.to_string());
                
                // Check if it has .git directory
                let git_status = if path.join(".git").exists() {
                    "✅ git"
                } else {
                    "❌ no git"
                };
                
                // Count snippets
                let snippet_count = count_snippets(&path)?;
                
                println!("  • {} ({}, {} snippets)", name, git_status, snippet_count);
            }
        }
    }
    
    if repos.is_empty() {
        println!("  (no repositories found)");
    }
    
    // Show current default
    println!();
    println!("🎯 Current default repository:");
    let config = config::Config::load()?;
    match config.get_default_repo() {
        Some(repo_name) => {
            let repo_path = repos_dir.join(repo_name);
            if repo_path.exists() {
                println!("  → {} ✅", repo_name);
            } else {
                println!("  → {} ⚠️  (directory missing)", repo_name);
            }
        }
        None => {
            println!("  → (not configured - will auto-detect)");
        }
    }
    
    println!();
    println!("📍 Repositories directory: {}", repos_dir.display());
    
    Ok(())
}

fn count_snippets(repo_path: &std::path::Path) -> Result<usize> {
    use std::fs;
    
    if !repo_path.exists() {
        return Ok(0);
    }
    
    let mut count = 0;
    for entry in fs::read_dir(repo_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            // Skip README.md and .gitignore type files
            if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                if !filename.eq_ignore_ascii_case("readme") {
                    count += 1;
                }
            }
        }
    }
    
    Ok(count)
}

async fn set_default_repo(repo_name: String) -> Result<()> {
    use std::fs;
    use publish::get_repos_dir;
    
    // Verify the repository exists
    let repos_dir = get_repos_dir()?;
    let repo_path = repos_dir.join(&repo_name);
    
    if !repo_path.exists() {
        println!("❌ Repository '{}' not found", repo_name);
        println!("📁 Available repositories:");
        
        if repos_dir.exists() {
            for entry in fs::read_dir(&repos_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        println!("  • {}", name);
                    }
                }
            }
        } else {
            println!("  (no repositories found - run 'claude-md-snippets setup')");
        }
        
        anyhow::bail!("Repository '{}' does not exist", repo_name);
    }
    
    // Set as default
    let mut config = config::Config::load()?;
    config.set_default_repo(repo_name.clone())?;
    
    println!("✅ Set '{}' as default repository", repo_name);
    
    Ok(())
}

async fn show_config() -> Result<()> {
    let config = config::Config::load()?;
    
    println!("⚙️  Claude MD Snippets Configuration");
    println!("===================================");
    
    match config.get_default_repo() {
        Some(repo_name) => {
            println!("🎯 Default repository: {}", repo_name);
            
            // Check if it exists
            let repos_dir = publish::get_repos_dir()?;
            let repo_path = repos_dir.join(repo_name);
            
            if repo_path.exists() {
                let snippet_count = count_snippets(&repo_path)?;
                let git_status = if repo_path.join(".git").exists() {
                    "✅ git initialized"
                } else {
                    "❌ no git"
                };
                println!("📊 Status: {} ({} snippets)", git_status, snippet_count);
            } else {
                println!("⚠️  Warning: Repository directory does not exist");
            }
        }
        None => {
            println!("🎯 Default repository: (not set)");
            println!("💡 Use 'claude-md-snippets config set-default <repo-name>' to set one");
        }
    }
    
    let config_path = publish::get_app_dir()?.join("config.json");
    println!("📍 Config file: {}", config_path.display());
    
    Ok(())
}