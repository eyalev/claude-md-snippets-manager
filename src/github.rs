use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;
use std::io::{self, Write};
use crate::publish::{Snippet, get_snippets_dir, get_repos_dir};

const DEFAULT_REPO: &str = "claude-md-snippets/community-snippets";

pub async fn sync_snippets() -> Result<()> {
    println!("🔄 Syncing snippets with GitHub repository...");
    
    let snippets_dir = get_snippets_dir()?;
    
    // Initialize git repository if it doesn't exist
    let git_dir = snippets_dir.join(".git");
    if !git_dir.exists() {
        println!("📦 Initializing snippet repository...");
        init_snippets_repo(&snippets_dir).await?;
    }
    
    // First, pull any remote changes
    println!("📥 Pulling latest changes from remote...");
    let pull_output = Command::new("git")
        .current_dir(&snippets_dir)
        .args(&["pull", "origin", "main"])
        .output()?;
    
    if !pull_output.status.success() {
        println!("⚠️  Warning: Could not pull from remote - continuing with local sync");
        let stderr = String::from_utf8_lossy(&pull_output.stderr);
        if !stderr.is_empty() && !stderr.contains("no such ref") {
            println!("⚠️  Git pull error: {}", stderr);
        }
    } else {
        println!("✅ Successfully pulled remote changes");
    }
    
    // Add all changes (including deletions)
    let output = Command::new("git")
        .current_dir(&snippets_dir)
        .args(&["add", "-A"])
        .output()?;
    
    if !output.status.success() {
        println!("⚠️  Warning: Could not stage changes");
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            println!("⚠️  Git error: {}", stderr);
        }
    }
    
    // Check if there are changes to commit
    let status_output = Command::new("git")
        .current_dir(&snippets_dir)
        .args(&["status", "--porcelain"])
        .output()?;
    
    if status_output.stdout.is_empty() {
        println!("✅ Sync complete - no local changes to push");
        return Ok(());
    }
    
    // Commit changes
    let commit_output = Command::new("git")
        .current_dir(&snippets_dir)
        .args(&["commit", "-m", "Sync snippets: add/modify/remove files"])
        .output()?;
    
    if !commit_output.status.success() {
        println!("⚠️  Warning: Could not create commit");
        return Ok(());
    }
    
    // Push to remote (if configured)
    println!("📤 Pushing to remote repository...");
    let push_output = Command::new("git")
        .current_dir(&snippets_dir)
        .args(&["push", "origin", "main"])
        .output();
    
    match push_output {
        Ok(output) if output.status.success() => {
            println!("✅ Successfully synced snippets! (pulled remote changes + pushed local changes)");
        }
        _ => {
            println!("⚠️  Could not push to remote. Make sure you have push access and the remote is configured.");
            println!("💡 To setup remote: cd {} && git remote add origin <your-repo-url>", snippets_dir.display());
        }
    }
    
    Ok(())
}

pub async fn pull_snippets() -> Result<()> {
    println!("📥 Pulling latest snippets from repository...");
    
    let snippets_dir = get_snippets_dir()?;
    
    if !snippets_dir.join(".git").exists() {
        println!("📦 Repository not initialized. Cloning default repository...");
        clone_default_repo().await?;
        return Ok(());
    }
    
    // Pull latest changes
    let output = Command::new("git")
        .current_dir(&snippets_dir)
        .args(&["pull", "origin", "main"])
        .output()?;
    
    if output.status.success() {
        println!("✅ Successfully pulled latest snippets!");
        
        // Show count of available snippets
        let snippets = load_snippets().await?;
        println!("📚 {} snippets available locally", snippets.len());
    } else {
        println!("⚠️  Could not pull from remote. Check your internet connection and repository configuration.");
    }
    
    Ok(())
}

async fn init_snippets_repo(snippets_dir: &std::path::Path) -> Result<()> {
    fs::create_dir_all(snippets_dir)?;
    
    // Initialize git repository
    Command::new("git")
        .current_dir(snippets_dir)
        .args(&["init"])
        .output()?;
    
    // Set default branch to main
    Command::new("git")
        .current_dir(snippets_dir)
        .args(&["branch", "-M", "main"])
        .output()?;
    
    // Create .gitignore
    let gitignore_content = "# Temp files\n*.tmp\n*.swp\n*~\n\n# OS files\n.DS_Store\nThumbs.db\n";
    fs::write(snippets_dir.join(".gitignore"), gitignore_content)?;
    
    // Create snippets directory
    fs::create_dir_all(snippets_dir.join("snippets"))?;
    
    // Create README
    let readme_content = "# Claude MD Snippets\n\nThis repository contains shared CLAUDE.md snippets.\n\n## Structure\n\n- `snippets/` - Contains all snippet files\n- Each snippet is stored as a markdown file with YAML frontmatter\n\n## Usage\n\nUse the `claude-md-snippets` CLI tool to publish, install, and search snippets.\n";
    fs::write(snippets_dir.join("README.md"), readme_content)?;
    
    // Configure git user for this repository
    configure_git_user(snippets_dir)?;
    
    // Initial commit
    Command::new("git")
        .current_dir(snippets_dir)
        .args(&["add", "."])
        .output()?;
    
    let commit_output = Command::new("git")
        .current_dir(snippets_dir)
        .args(&["commit", "-m", "Initial commit"])
        .output()?;
    
    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        println!("⚠️  Warning: Could not create initial commit: {}", stderr);
    }
    
    println!("✅ Initialized local snippet repository");
    println!("💡 To sync with a remote repository, add a remote:");
    println!("   cd {} && git remote add origin <your-repo-url>", snippets_dir.display());
    
    Ok(())
}

async fn clone_default_repo() -> Result<()> {
    let snippets_dir = get_snippets_dir()?;
    let parent_dir = snippets_dir.parent().unwrap();
    
    // Remove existing directory if it exists
    if snippets_dir.exists() {
        fs::remove_dir_all(&snippets_dir)?;
    }
    
    // Clone the repository
    let output = Command::new("git")
        .current_dir(parent_dir)
        .args(&[
            "clone", 
            &format!("https://github.com/{}", DEFAULT_REPO),
            snippets_dir.file_name().unwrap().to_str().unwrap()
        ])
        .output()?;
    
    if output.status.success() {
        println!("✅ Cloned community snippets repository");
    } else {
        println!("⚠️  Could not clone default repository. Creating local repository instead.");
        init_snippets_repo(&snippets_dir).await?;
    }
    
    Ok(())
}

async fn load_snippets() -> Result<Vec<Snippet>> {
    let repo_dir = get_snippets_dir()?;
    let snippets_dir = repo_dir.join("snippets");
    
    if !snippets_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut snippets = Vec::new();
    
    for entry in fs::read_dir(snippets_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(snippet) = crate::publish::parse_markdown_frontmatter(&content) {
                    snippets.push(snippet);
                }
            }
        }
    }
    
    Ok(snippets)
}

pub async fn setup_repository(repo_name_option: Option<String>) -> Result<()> {
    println!("🔧 Setting up GitHub repository for claude-md-snippets...");
    
    // Check if gh CLI is available
    let gh_check = Command::new("gh").arg("--version").output();
    
    let use_gh_cli = match gh_check {
        Ok(output) if output.status.success() => {
            println!("✅ GitHub CLI detected");
            true
        }
        _ => {
            println!("⚠️  GitHub CLI not found. You'll need to create the repository manually.");
            false
        }
    };
    
    // Get repository visibility and name from user
    let (is_private, github_repo_name) = if let Some(provided_name) = &repo_name_option {
        // If repo name is provided, assume private for backward compatibility
        (true, provided_name.clone())
    } else {
        // Ask for visibility
        print!("Create repository as private or public? (p/P for public, default: private): ");
        io::stdout().flush()?;
        let mut visibility = String::new();
        io::stdin().read_line(&mut visibility)?;
        let visibility = visibility.trim().to_lowercase();
        
        let is_private = !matches!(visibility.as_str(), "p" | "public");
        
        // Set default name based on visibility
        let default_name = if is_private {
            "claude-md-snippets-private"
        } else {
            "claude-md-snippets"
        };
        
        let repo_type = if is_private { "private" } else { "public" };
        print!("Enter GitHub repository name (default: {}): ", default_name);
        io::stdout().flush()?;
        let mut repo_name = String::new();
        io::stdin().read_line(&mut repo_name)?;
        let repo_name = repo_name.trim();
        
        let final_name = if repo_name.is_empty() { 
            default_name.to_string()
        } else { 
            repo_name.to_string()
        };
        
        println!("Creating {} repository '{}'", repo_type, final_name);
        
        (is_private, final_name)
    };
    
    // Use the same name for local directory
    let repos_dir = get_repos_dir()?;
    let snippets_dir = repos_dir.join(&github_repo_name);
    
    if use_gh_cli {
        // Create repository using gh CLI
        let visibility_flag = if is_private { "--private" } else { "--public" };
        let visibility_text = if is_private { "private" } else { "public" };
        println!("📦 Creating {} repository '{}'...", visibility_text, github_repo_name);
        
        let create_output = Command::new("gh")
            .args(&["repo", "create", &github_repo_name, visibility_flag, "--description", "Personal CLAUDE.md snippets"])
            .output()?;
        
        if !create_output.status.success() {
            let stderr = String::from_utf8_lossy(&create_output.stderr);
            if stderr.contains("already exists") {
                println!("ℹ️  Repository '{}' already exists", github_repo_name);
            } else {
                println!("⚠️  Failed to create repository: {}", stderr);
                return manual_setup_instructions(&github_repo_name, &snippets_dir, is_private);
            }
        } else {
            println!("✅ Repository created successfully!");
        }
        
        // Initialize local repository if needed
        if !snippets_dir.join(".git").exists() {
            init_snippets_repo(&snippets_dir).await?;
        }
        
        // Add remote
        let username = get_github_username()?;
        let remote_url = format!("https://github.com/{}/{}.git", username, github_repo_name);
        
        println!("🔗 Adding remote origin...");
        let remote_output = Command::new("git")
            .current_dir(&snippets_dir)
            .args(&["remote", "add", "origin", &remote_url])
            .output();
        
        match remote_output {
            Ok(output) if output.status.success() => {
                println!("✅ Remote origin added: {}", remote_url);
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("already exists") {
                    // Update existing remote
                    Command::new("git")
                        .current_dir(&snippets_dir)
                        .args(&["remote", "set-url", "origin", &remote_url])
                        .output()?;
                    println!("✅ Remote origin updated: {}", remote_url);
                } else {
                    println!("⚠️  Could not add remote: {}", stderr);
                }
            }
            Err(e) => {
                println!("⚠️  Error adding remote: {}", e);
            }
        }
        
        // Try initial push, but handle existing repository case
        println!("📤 Pushing to remote repository...");
        let push_output = Command::new("git")
            .current_dir(&snippets_dir)
            .args(&["push", "-u", "origin", "main"])
            .output()?;
        
        if push_output.status.success() {
            println!("✅ Setup complete! Your snippets repository is ready.");
            println!("🌐 Repository: https://github.com/{}/{}", username, github_repo_name);
            println!("📁 Local directory: {}", snippets_dir.display());
        } else {
            let stderr = String::from_utf8_lossy(&push_output.stderr);
            
            // Check if this is a "fetch first" error indicating existing remote content
            if stderr.contains("rejected") && stderr.contains("fetch first") {
                println!("📥 Repository already has content. Syncing with remote...");
                
                // Try to pull and merge with explicit merge strategy
                let pull_output = Command::new("git")
                    .current_dir(&snippets_dir)
                    .args(&["pull", "origin", "main", "--allow-unrelated-histories", "--no-rebase"])
                    .output()?;
                
                if pull_output.status.success() {
                    println!("✅ Successfully synced with existing repository content.");
                    
                    // Try push again
                    let retry_push = Command::new("git")
                        .current_dir(&snippets_dir)
                        .args(&["push", "-u", "origin", "main"])
                        .output()?;
                    
                    if retry_push.status.success() {
                        println!("✅ Setup complete! Your snippets repository is ready.");
                        println!("🌐 Repository: https://github.com/{}/{}", username, github_repo_name);
                        println!("📁 Local directory: {}", snippets_dir.display());
                    } else {
                        println!("⚠️  Could not push after sync. Manual intervention may be needed.");
                        println!("💡 Try running 'claude-md-snippets sync' to resolve any conflicts");
                    }
                } else {
                    let pull_stderr = String::from_utf8_lossy(&pull_output.stderr);
                    println!("⚠️  Could not sync with existing repository: {}", pull_stderr);
                    println!("💡 You may need to manually resolve conflicts in: {}", snippets_dir.display());
                }
            } else {
                println!("⚠️  Push failed: {}", stderr);
                println!("💡 Try running 'claude-md-snippets sync' after creating some snippets");
            }
        }
        
        // Set as default repository regardless of push success
        let mut config = crate::config::Config::load()?;
        config.set_default_repo(github_repo_name.clone())?;
        println!("🎯 Set '{}' as your default repository", github_repo_name);
        
    } else {
        manual_setup_instructions(&github_repo_name, &snippets_dir, is_private)?;
        
        // Initialize local repository for manual setup too
        if !snippets_dir.join(".git").exists() {
            init_snippets_repo(&snippets_dir).await?;
        }
        
        // Set as default repository
        let mut config = crate::config::Config::load()?;
        config.set_default_repo(github_repo_name.clone())?;
        println!("🎯 Set '{}' as your default repository", github_repo_name);
    }
    
    Ok(())
}

fn manual_setup_instructions(repo_name: &str, snippets_dir: &std::path::Path, is_private: bool) -> Result<()> {
    let visibility = if is_private { "private" } else { "public" };
    println!("\n📝 Manual Setup Instructions:");
    println!("1. Create a new {} repository on GitHub named '{}'", visibility, repo_name);
    println!("2. Run the following commands:");
    println!("   cd {}", snippets_dir.display());
    println!("   git remote add origin https://github.com/YOUR_USERNAME/{}.git", repo_name);
    println!("   git push -u origin main");
    println!("\n💡 After setup, use 'claude-md-snippets sync' to upload snippets");
    Ok(())
}

fn configure_git_user(snippets_dir: &std::path::Path) -> Result<()> {
    // Check if git is already configured globally
    let global_name = Command::new("git")
        .args(&["config", "--global", "user.name"])
        .output();
    
    let global_email = Command::new("git")
        .args(&["config", "--global", "user.email"])
        .output();
    
    // If global config exists, use it
    if let (Ok(name_output), Ok(email_output)) = (&global_name, &global_email) {
        if name_output.status.success() && email_output.status.success() {
            return Ok(()); // Global config exists, we're good
        }
    }
    
    // Try to get info from GitHub CLI
    let gh_user = Command::new("gh")
        .args(&["api", "user"])
        .output();
    
    let (username, email) = if let Ok(output) = gh_user {
        if output.status.success() {
            let user_info = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&user_info) {
                let username = json["login"].as_str().unwrap_or("claude-snippets-user").to_string();
                let email = json["email"].as_str()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| format!("{}@users.noreply.github.com", username));
                (username, email)
            } else {
                ("claude-snippets-user".to_string(), "claude-snippets-user@users.noreply.github.com".to_string())
            }
        } else {
            ("claude-snippets-user".to_string(), "claude-snippets-user@users.noreply.github.com".to_string())
        }
    } else {
        ("claude-snippets-user".to_string(), "claude-snippets-user@users.noreply.github.com".to_string())
    };
    
    // Configure for this repository only
    Command::new("git")
        .current_dir(snippets_dir)
        .args(&["config", "user.name", &username])
        .output()?;
    
    Command::new("git")
        .current_dir(snippets_dir)
        .args(&["config", "user.email", &email])
        .output()?;
    
    Ok(())
}

fn get_github_username() -> Result<String> {
    // Try to get username from gh CLI
    let output = Command::new("gh")
        .args(&["api", "user", "--jq", ".login"])
        .output()?;
    
    if output.status.success() {
        let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(username);
    }
    
    // Fallback: ask user
    print!("Enter your GitHub username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    Ok(username.trim().to_string())
}