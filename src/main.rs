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
    /// Enable debug logging
    #[arg(long, global = true)]
    debug: bool,
    
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
    /// Install a snippet to CLAUDE.md
    Install {
        /// Description to find the relevant snippet
        query: String,
        /// Install to local CLAUDE.md in current directory
        #[arg(long, conflicts_with = "user")]
        local: bool,
        /// Install to user CLAUDE.md at ~/.claude/CLAUDE.md
        #[arg(long, conflicts_with = "local")]
        user: bool,
    },
    /// Uninstall a snippet from CLAUDE.md
    Uninstall {
        /// Description or ID to find the snippet to uninstall
        query: String,
        /// Uninstall from local CLAUDE.md in current directory
        #[arg(long, conflicts_with = "user")]
        local: bool,
        /// Uninstall from user CLAUDE.md at ~/.claude/CLAUDE.md
        #[arg(long, conflicts_with = "local")]
        user: bool,
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
    /// Manage repository content
    Repo {
        /// Repository name (defaults to configured default)
        #[arg(long)]
        name: Option<String>,
        /// Use default repository
        #[arg(long)]
        default: bool,
        #[command(subcommand)]
        repo_command: RepoCommand,
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
    /// Set the default install location
    SetInstallLocation {
        /// Install location: 'local' or 'user'
        location: String,
    },
}

#[derive(Subcommand)]
enum RepoCommand {
    /// Delete a snippet from the repository
    Delete {
        /// Description or query to find the snippet to delete
        query: String,
    },
    /// List snippets in the repository
    List,
    /// Open repository in browser
    Open,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Publish { content, name, file } => {
            publish::publish_snippet(content, name, file, cli.debug).await?;
        }
        Commands::Install { query, local, user } => {
            install::install_snippet(query, local, user).await?;
        }
        Commands::Uninstall { query, local, user } => {
            install::uninstall_snippet(query, local, user).await?;
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
                ConfigCommand::SetInstallLocation { location } => {
                    set_install_location(location).await?;
                }
            }
        }
        Commands::Repo { name, default, repo_command } => {
            match repo_command {
                RepoCommand::Delete { query } => {
                    delete_snippet(name, default, query, cli.debug).await?;
                }
                RepoCommand::List => {
                    list_repo_snippets(name, default).await?;
                }
                RepoCommand::Open => {
                    open_repo_in_browser(name, default).await?;
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
    
    let snippets_dir = repo_path.join("snippets");
    if !snippets_dir.exists() {
        return Ok(0);
    }
    
    let mut count = 0;
    for entry in fs::read_dir(snippets_dir)? {
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
    println!("📍 Default install location: {}", config.get_default_install_location());
    
    Ok(())
}

async fn set_install_location(location: String) -> Result<()> {
    let mut config = config::Config::load()?;
    
    match config.set_default_install_location(location.clone()) {
        Ok(()) => {
            println!("✅ Set default install location to: {}", location);
            match location.as_str() {
                "local" => println!("💡 Snippets will install to ./CLAUDE.md by default"),
                "user" => println!("💡 Snippets will install to ~/.claude/CLAUDE.md by default"),
                _ => {}
            }
        }
        Err(e) => {
            println!("❌ Failed to set install location: {}", e);
            println!("💡 Valid options are: 'local' or 'user'");
        }
    }
    
    Ok(())
}

async fn delete_snippet(repo_name: Option<String>, use_default: bool, query: String, debug: bool) -> Result<()> {
    use std::fs;
    use std::process::Command;
    use std::io::{self, Write};
    use publish::get_repos_dir;
    
    // Determine which repository to use
    let target_repo = if use_default || repo_name.is_none() {
        config::get_default_repo_name()?
    } else {
        repo_name.unwrap()
    };
    
    let repos_dir = get_repos_dir()?;
    let repo_dir = repos_dir.join(&target_repo);
    
    if !repo_dir.exists() {
        anyhow::bail!("Repository '{}' not found at {}", target_repo, repo_dir.display());
    }
    
    println!("🔍 Searching for snippet matching '{}' in repository '{}'...", query, target_repo);
    
    // Find the file using intelligent matching (in snippets subdirectory)
    let snippets_subdir = repo_dir.join("snippets");
    if !snippets_subdir.exists() {
        fs::create_dir_all(&snippets_subdir)?;
    }
    let file_to_delete = find_snippet_file_intelligently(&query, &snippets_subdir, debug)?;
    
    // Read the file to show what will be deleted
    let content = fs::read_to_string(&file_to_delete)?;
    let snippet_info = if let Ok(snippet) = publish::parse_markdown_frontmatter(&content) {
        format!("'{}' (ID: {})", snippet.name, &snippet.id[..8])
    } else {
        file_to_delete.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    };
    
    // Confirm deletion
    println!("📄 Found snippet: {}", snippet_info);
    println!("📁 File: {}", file_to_delete.display());
    print!("❓ Are you sure you want to delete this snippet? (y/N): ");
    std::io::stdout().flush()?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    if input != "y" && input != "yes" {
        println!("❌ Deletion cancelled");
        return Ok(());
    }
    
    // Delete the file
    fs::remove_file(&file_to_delete)?;
    println!("✅ Deleted snippet: {}", snippet_info);
    
    // Auto-sync with repository
    println!("🔄 Syncing deletion with repository...");
    match crate::github::sync_snippets().await {
        Ok(()) => {
            println!("✅ Successfully synced deletion to repository!");
        }
        Err(e) => {
            println!("⚠️  Sync failed: {}", e);
            println!("💡 You can manually sync later with 'claude-md-snippets sync'");
        }
    }
    
    Ok(())
}

fn find_snippet_file_intelligently(query: &str, repo_dir: &std::path::Path, debug: bool) -> Result<std::path::PathBuf> {
    use std::fs;
    use std::process::Command;
    
    // First try simple filename matching
    let mut simple_matches = Vec::new();
    for entry in fs::read_dir(repo_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Skip README and similar files
                if filename.to_lowercase().contains("readme") {
                    continue;
                }
                
                if filename.to_lowercase().contains(&query.to_lowercase()) {
                    simple_matches.push(path);
                }
            }
        }
    }
    
    if simple_matches.len() == 1 {
        return Ok(simple_matches[0].clone());
    }
    
    // Use Claude Code for intelligent matching
    println!("🤔 Using intelligent search to find matching snippet...");
    
    // Get list of all snippet files with content preview
    let mut file_list = String::new();
    for entry in fs::read_dir(repo_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Skip README and similar files
                if filename.to_lowercase().contains("readme") {
                    continue;
                }
                
                // Read and preview the file
                let content = fs::read_to_string(&path).unwrap_or_default();
                let preview = if let Ok(snippet) = publish::parse_markdown_frontmatter(&content) {
                    format!("Name: {}\nContent preview:\n{}", 
                        snippet.name,
                        snippet.content.lines().take(5).collect::<Vec<_>>().join("\n")
                    )
                } else {
                    content.lines().take(10).collect::<Vec<_>>().join("\n")
                };
                
                file_list.push_str(&format!(
                    "File: {}\n{}\n\n---\n\n",
                    filename,
                    preview
                ));
            }
        }
    }
    
    if file_list.is_empty() {
        anyhow::bail!("No markdown snippet files found in repository '{}'", repo_dir.display());
    }
    
    // Use Claude Code to find the best match
    let prompt = format!(
        "Based on the query '{}', which file from the list below is the best match? \
        Just respond with the exact filename (including extension), nothing else.\n\n{}",
        query, file_list
    );
    
    if debug {
        println!("🔧 Debug: Calling Claude Code CLI...");
        println!("🔧 Debug: Command: claude --dangerously-skip-permissions --print <prompt>");
        println!("🔧 Debug: Prompt length: {} characters", prompt.len());
    }
    
    let output = std::process::Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--print")
        .arg(&prompt)
        .output();
    
    let output = match output {
        Ok(output) => {
            if debug {
                println!("🔧 Debug: Claude Code CLI returned with status: {}", output.status);
                if !output.stderr.is_empty() {
                    println!("🔧 Debug: stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            output
        }
        Err(e) => {
            println!("⚠️  Failed to execute Claude Code CLI: {}", e);
            println!("💡 Falling back to simple matching");
            // Fallback to simple matching
            if simple_matches.len() > 1 {
                println!("⚠️  Multiple matches found:");
                for (i, file) in simple_matches.iter().enumerate() {
                    println!("  {}. {}", i + 1, file.display());
                }
                anyhow::bail!("Please be more specific with your query");
            } else if simple_matches.is_empty() {
                anyhow::bail!("No snippet found matching '{}' in repository", query);
            }
            return Ok(simple_matches[0].clone());
        }
    };
    
    if !output.status.success() {
        // Fallback to simple matching if Claude Code fails
        if simple_matches.len() > 1 {
            println!("⚠️  Claude Code unavailable. Multiple matches found:");
            for (i, file) in simple_matches.iter().enumerate() {
                println!("  {}. {}", i + 1, file.display());
            }
            anyhow::bail!("Please be more specific with your query");
        } else if simple_matches.is_empty() {
            anyhow::bail!("No snippet found matching '{}' in repository", query);
        }
        return Ok(simple_matches[0].clone());
    }
    
    let suggested_filename = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let suggested_path = repo_dir.join(&suggested_filename);
    
    if suggested_path.exists() {
        Ok(suggested_path)
    } else {
        anyhow::bail!("Suggested file '{}' not found in repository", suggested_filename);
    }
}

async fn list_repo_snippets(repo_name: Option<String>, use_default: bool) -> Result<()> {
    use std::fs;
    use publish::get_repos_dir;
    
    // Determine which repository to use
    let target_repo = if use_default || repo_name.is_none() {
        config::get_default_repo_name()?
    } else {
        repo_name.unwrap()
    };
    
    let repos_dir = get_repos_dir()?;
    let repo_dir = repos_dir.join(&target_repo);
    
    if !repo_dir.exists() {
        anyhow::bail!("Repository '{}' not found at {}", target_repo, repo_dir.display());
    }
    
    println!("📚 Snippets in repository '{}':", target_repo);
    println!("================================");
    
    let mut snippets = Vec::new();
    
    // Look in snippets subdirectory
    let snippets_subdir = repo_dir.join("snippets");
    if !snippets_subdir.exists() {
        println!("  (no snippets directory found)");
        return Ok(());
    }
    
    for entry in fs::read_dir(&snippets_subdir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Skip README and similar files
                if filename.to_lowercase().contains("readme") {
                    continue;
                }
                
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(snippet) = publish::parse_markdown_frontmatter(&content) {
                        snippets.push((filename.to_string(), snippet));
                    } else {
                        // File without frontmatter
                        snippets.push((filename.to_string(), publish::Snippet {
                            id: "unknown".to_string(),
                            name: filename.replace(".md", "").replace("_", " "),
                            content: content,
                            created_at: "unknown".to_string(),
                            description: None,
                        }));
                    }
                }
            }
        }
    }
    
    if snippets.is_empty() {
        println!("  (no snippets found)");
    } else {
        // Sort by creation date (newest first)
        snippets.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
        
        for (filename, snippet) in snippets {
            let created = if snippet.created_at != "unknown" {
                chrono::DateTime::parse_from_rfc3339(&snippet.created_at)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|_| snippet.created_at)
            } else {
                "unknown".to_string()
            };
            
            println!("  📄 {} ({})", snippet.name, &snippet.id[..8]);
            println!("      File: {}", filename);
            println!("      Created: {}", created);
            if let Some(desc) = &snippet.description {
                println!("      Description: {}", desc);
            }
            println!();
        }
    }
    
    println!("📍 Repository directory: {}", repo_dir.display());
    
    Ok(())
}

async fn open_repo_in_browser(repo_name: Option<String>, use_default: bool) -> Result<()> {
    use std::process::Command;
    use publish::get_repos_dir;
    
    // Determine which repository to use
    let target_repo = if use_default || repo_name.is_none() {
        config::get_default_repo_name()?
    } else {
        repo_name.unwrap()
    };
    
    let repos_dir = get_repos_dir()?;
    let repo_dir = repos_dir.join(&target_repo);
    
    if !repo_dir.exists() {
        anyhow::bail!("Repository '{}' not found at {}", target_repo, repo_dir.display());
    }
    
    // Check if this is a git repository
    let git_dir = repo_dir.join(".git");
    if !git_dir.exists() {
        anyhow::bail!("Repository '{}' is not a git repository. Initialize with git first.", target_repo);
    }
    
    // Get the remote URL
    let output = Command::new("git")
        .current_dir(&repo_dir)
        .args(&["remote", "get-url", "origin"])
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("No git remote 'origin' found for repository '{}'. Add a remote first.", target_repo);
    }
    
    let remote_url = String::from_utf8(output.stdout)?.trim().to_string();
    
    // Convert git URL to HTTPS URL if needed
    let browser_url = if remote_url.starts_with("git@github.com:") {
        remote_url.replace("git@github.com:", "https://github.com/")
            .strip_suffix(".git").unwrap_or(&remote_url).to_string()
    } else if remote_url.starts_with("https://github.com/") {
        remote_url.strip_suffix(".git").unwrap_or(&remote_url).to_string()
    } else {
        remote_url
    };
    
    println!("🌐 Opening repository '{}' in browser...", target_repo);
    println!("🔗 URL: {}", browser_url);
    
    // Open URL in default browser
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(&browser_url).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/c", "start", &browser_url]).status()
    } else {
        // Linux and other Unix-like systems
        Command::new("xdg-open").arg(&browser_url).status()
    };
    
    match result {
        Ok(status) if status.success() => {
            println!("✅ Successfully opened repository in browser");
        }
        Ok(_) => {
            println!("⚠️  Failed to open browser. You can manually visit: {}", browser_url);
        }
        Err(e) => {
            println!("⚠️  Failed to open browser ({}). You can manually visit: {}", e, browser_url);
        }
    }
    
    Ok(())
}