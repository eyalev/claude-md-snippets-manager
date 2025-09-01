use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: String,
    pub description: Option<String>,
}

pub async fn publish_snippet(content: Option<String>, custom_name: Option<String>, file: Option<String>, debug: bool) -> Result<()> {
    // Determine content source and create snippet
    let snippet = if let Some(file_query) = file {
        // Load from extracted snippet file and preserve original metadata
        load_snippet_from_local_file(&file_query, custom_name, debug)?
    } else if let Some(content_str) = content {
        // Create new snippet from content
        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();
        let name = if let Some(name) = custom_name {
            name
        } else {
            generate_name_from_content(&content_str)
        };
        
        Snippet {
            id,
            name,
            content: content_str,
            created_at: timestamp,
            description: None,
        }
    } else {
        anyhow::bail!("Either content or --file must be provided");
    };

    // Ensure directory structure exists (with snippets subdirectory)
    let repo_dir = get_snippets_dir()?;
    let snippets_dir = repo_dir.join("snippets");
    fs::create_dir_all(&snippets_dir)?;
    
    let filename = format!("{}-{}.md", snippet.name.replace(' ', "-").to_lowercase(), &snippet.id[..8]);
    let filepath = snippets_dir.join(filename);
    
    let markdown_content = create_markdown_with_frontmatter(&snippet)?;
    fs::write(&filepath, markdown_content)?;
    
    println!("✅ Published snippet '{}' (ID: {})", snippet.name, snippet.id);
    println!("📁 Saved to: {}", filepath.display());
    
    // Automatically sync with repository
    println!("🔄 Syncing with repository...");
    match crate::github::sync_snippets().await {
        Ok(()) => {
            println!("✅ Successfully synced to repository!");
        }
        Err(e) => {
            println!("⚠️  Sync failed: {}", e);
            println!("💡 You can manually sync later with 'claude-md-snippets sync'");
        }
    }
    
    Ok(())
}

fn load_snippet_from_local_file(file_query: &str, custom_name: Option<String>, debug: bool) -> Result<Snippet> {
    use std::path::Path;
    use std::process::Command;
    
    // Look for snippet file in ./.claude.local/snippets/
    let local_snippets_dir = Path::new("./.claude.local/snippets");
    
    if !local_snippets_dir.exists() {
        anyhow::bail!("No local snippets directory found. Run 'claude-md-snippets extract' first.");
    }
    
    // First try simple matching as fallback
    let mut simple_matches = Vec::new();
    for entry in fs::read_dir(local_snippets_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.to_lowercase().contains(&file_query.to_lowercase()) {
                simple_matches.push(path);
            }
        }
    }
    
    // If simple matching works and finds exactly one file, use it
    if simple_matches.len() == 1 {
        let file_path = &simple_matches[0];
        let content = fs::read_to_string(file_path)?;
        println!("📖 Found matching file: {}", file_path.display());
        
        // Try to parse existing frontmatter to preserve metadata
        if let Ok(existing_snippet) = parse_markdown_frontmatter(&content) {
            let final_name = if let Some(custom) = custom_name {
                custom
            } else {
                existing_snippet.name
            };
            
            return Ok(Snippet {
                id: existing_snippet.id,
                name: final_name,
                content: existing_snippet.content,
                created_at: existing_snippet.created_at,
                description: existing_snippet.description,
            });
        } else {
            // Fallback for files without frontmatter
            let name = get_name_from_file(file_path, &custom_name)?;
            let id = Uuid::new_v4().to_string();
            let timestamp = chrono::Utc::now().to_rfc3339();
            
            return Ok(Snippet {
                id,
                name,
                content,
                created_at: timestamp,
                description: None,
            });
        }
    }
    
    // Use Claude Code for intelligent matching
    println!("🤔 Using intelligent search to find matching snippet...");
    let matched_file = find_file_with_claude_code(file_query, local_snippets_dir, debug)?;
    
    let content = fs::read_to_string(&matched_file)?;
    
    println!("📖 Found matching file: {}", matched_file.display());
    
    // Try to parse existing frontmatter to preserve metadata
    if let Ok(existing_snippet) = parse_markdown_frontmatter(&content) {
        // Use existing snippet but allow custom name override
        let final_name = if let Some(custom) = custom_name {
            custom
        } else {
            existing_snippet.name
        };
        
        Ok(Snippet {
            id: existing_snippet.id,
            name: final_name,
            content: existing_snippet.content,
            created_at: existing_snippet.created_at,
            description: existing_snippet.description,
        })
    } else {
        // Fallback: create new snippet if parsing fails
        let name = get_name_from_file(&matched_file, &custom_name)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        Ok(Snippet {
            id,
            name,
            content,
            created_at: timestamp,
            description: None,
        })
    }
}

fn find_file_with_claude_code(query: &str, snippets_dir: &Path, debug: bool) -> Result<std::path::PathBuf> {
    use std::process::Command;
    
    // Get list of all files in the directory
    let mut file_list = String::new();
    for entry in fs::read_dir(snippets_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Read first few lines to get an idea of content
                let preview = fs::read_to_string(&path)
                    .unwrap_or_default()
                    .lines()
                    .take(10)
                    .collect::<Vec<_>>()
                    .join("\n");
                
                file_list.push_str(&format!(
                    "File: {}\nPreview:\n{}\n\n---\n\n",
                    filename,
                    preview
                ));
            }
        }
    }
    
    if file_list.is_empty() {
        anyhow::bail!("No markdown files found in ./.claude.local/snippets/");
    }
    
    // Use Claude Code to find the best match
    let prompt = format!(
        "Based on the query '{}', which file from the list below is the best match? \
        Just respond with the exact filename (including extension), nothing else.\n\n{}",
        query, file_list
    );
    
    if debug {
        println!("🔧 Debug: Calling Claude Code CLI for file matching...");
        println!("🔧 Debug: Command: claude --dangerously-skip-permissions --print <prompt>");
        println!("🔧 Debug: Prompt length: {} characters", prompt.len());
    }
    
    let output = Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--print")
        .arg(&prompt)
        .output()?;
    
    if debug {
        println!("🔧 Debug: Claude Code CLI returned with status: {}", output.status);
        if !output.stderr.is_empty() {
            println!("🔧 Debug: stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
    
    if !output.status.success() {
        // Fallback to simple matching if Claude Code fails
        println!("⚠️  Claude Code unavailable, falling back to simple matching");
        return simple_fallback_match(query, snippets_dir);
    }
    
    let suggested_filename = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let suggested_path = snippets_dir.join(&suggested_filename);
    
    if suggested_path.exists() {
        Ok(suggested_path)
    } else {
        // Claude might have suggested something that doesn't exist exactly, try fallback
        println!("⚠️  Suggested file '{}' not found, trying fallback matching", suggested_filename);
        simple_fallback_match(query, snippets_dir)
    }
}

fn simple_fallback_match(query: &str, snippets_dir: &Path) -> Result<std::path::PathBuf> {
    let mut matches = Vec::new();
    
    for entry in fs::read_dir(snippets_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.to_lowercase().contains(&query.to_lowercase()) {
                matches.push(path);
            }
        }
    }
    
    if matches.is_empty() {
        anyhow::bail!("No snippet file found matching '{}' in ./.claude.local/snippets/", query);
    }
    
    if matches.len() > 1 {
        println!("Multiple files found:");
        for file in &matches {
            println!("  - {}", file.display());
        }
        anyhow::bail!("Please be more specific with the file query.");
    }
    
    Ok(matches[0].clone())
}

fn get_name_from_file(file_path: &Path, custom_name: &Option<String>) -> Result<String> {
    let name = if let Some(custom) = custom_name {
        custom.clone()
    } else if let Some(filename) = file_path.file_stem().and_then(|n| n.to_str()) {
        // Convert filename back to readable format, removing the ID suffix
        let name_part = filename.split('-').collect::<Vec<_>>();
        let clean_name = if name_part.len() > 1 {
            // Remove the last part (ID) and rejoin
            name_part[..name_part.len()-1].join("-")
        } else {
            filename.to_string()
        };
        clean_name.replace('_', " ")
    } else {
        "Untitled".to_string()
    };
    
    Ok(name)
}

fn generate_name_from_content(content: &str) -> String {
    // Extract first meaningful line or generate from keywords
    let lines: Vec<&str> = content.lines().collect();
    
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            // Take first few words
            let words: Vec<&str> = trimmed.split_whitespace().take(4).collect();
            if !words.is_empty() {
                return words.join(" ");
            }
        }
    }
    
    // If we can't extract meaningful content, look for markdown headers
    for line in content.lines() {
        if line.starts_with('#') {
            let header = line.trim_start_matches('#').trim();
            if !header.is_empty() {
                return header.to_string();
            }
        }
    }
    
    // Fallback to generic name with timestamp
    format!("snippet-{}", chrono::Utc::now().format("%Y%m%d-%H%M"))
}

fn create_markdown_with_frontmatter(snippet: &Snippet) -> Result<String> {
    // Create frontmatter
    let frontmatter = format!(
        "---\nid: {}\nname: {}\ncreated_at: {}\ndescription: {}\n---\n\n",
        snippet.id,
        snippet.name,
        snippet.created_at,
        snippet.description.as_deref().unwrap_or("null")
    );
    
    // Combine frontmatter with content
    let full_content = format!("{}{}", frontmatter, snippet.content);
    
    Ok(full_content)
}

pub fn parse_markdown_frontmatter(content: &str) -> Result<Snippet> {
    // Split frontmatter from content
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    
    if parts.len() < 3 {
        anyhow::bail!("Invalid markdown format: missing frontmatter");
    }
    
    let frontmatter_yaml = parts[1].trim();
    let markdown_content = parts[2].trim_start_matches('\n');
    
    // Parse YAML frontmatter
    let frontmatter: serde_yaml::Value = serde_yaml::from_str(frontmatter_yaml)?;
    
    let snippet = Snippet {
        id: frontmatter["id"].as_str().unwrap_or("").to_string(),
        name: frontmatter["name"].as_str().unwrap_or("").to_string(),
        created_at: frontmatter["created_at"].as_str().unwrap_or("").to_string(),
        description: match frontmatter["description"].as_str() {
            Some("null") | None => None,
            Some(desc) => Some(desc.to_string()),
        },
        content: markdown_content.to_string(),
    };
    
    Ok(snippet)
}

pub fn get_app_dir() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".claude-md-snippets"))
}

pub fn get_repos_dir() -> Result<std::path::PathBuf> {
    let app_dir = get_app_dir()?;
    Ok(app_dir.join("repos"))
}

pub fn get_default_repo_dir() -> Result<std::path::PathBuf> {
    let repos_dir = get_repos_dir()?;
    let default_repo_name = crate::config::get_default_repo_name()?;
    Ok(repos_dir.join(default_repo_name))
}

// Backward compatibility - use default repo
pub fn get_snippets_dir() -> Result<std::path::PathBuf> {
    get_default_repo_dir()
}