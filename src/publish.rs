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

pub async fn publish_snippet(content: Option<String>, custom_name: Option<String>, file: Option<String>) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();
    
    // Determine content source and name
    let (snippet_content, snippet_name) = if let Some(file_query) = file {
        // Load from extracted snippet file
        load_from_local_snippet(&file_query, custom_name)?
    } else if let Some(content_str) = content {
        // Use provided content
        let name = if let Some(name) = custom_name {
            name
        } else {
            generate_name_from_content(&content_str)
        };
        (content_str, name)
    } else {
        anyhow::bail!("Either content or --file must be provided");
    };

    let snippet = Snippet {
        id: id.clone(),
        name: snippet_name.clone(),
        content: snippet_content,
        created_at: timestamp,
        description: None,
    };

    // Ensure directory structure exists
    let snippets_dir = get_snippets_dir()?;
    fs::create_dir_all(&snippets_dir)?;
    
    let filename = format!("{}-{}.md", snippet_name.replace(' ', "-").to_lowercase(), &id[..8]);
    let filepath = snippets_dir.join(filename);
    
    let markdown_content = create_markdown_with_frontmatter(&snippet)?;
    fs::write(&filepath, markdown_content)?;
    
    println!("✅ Published snippet '{}' (ID: {})", snippet_name, id);
    println!("📁 Saved to: {}", filepath.display());
    
    Ok(())
}

fn load_from_local_snippet(file_query: &str, custom_name: Option<String>) -> Result<(String, String)> {
    use std::path::Path;
    
    // Look for snippet file in ./.claude.local/snippets/
    let local_snippets_dir = Path::new("./.claude.local/snippets");
    
    if !local_snippets_dir.exists() {
        anyhow::bail!("No local snippets directory found. Run 'claude-md-snippets extract' first.");
    }
    
    // Try to find matching file
    let mut matching_files = Vec::new();
    
    for entry in fs::read_dir(local_snippets_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.to_lowercase().contains(&file_query.to_lowercase()) {
                matching_files.push(path);
            }
        }
    }
    
    if matching_files.is_empty() {
        anyhow::bail!("No snippet file found matching '{}' in ./.claude.local/snippets/", file_query);
    }
    
    if matching_files.len() > 1 {
        println!("Multiple files found:");
        for file in &matching_files {
            println!("  - {}", file.display());
        }
        anyhow::bail!("Please be more specific with the file query.");
    }
    
    let file_path = &matching_files[0];
    let content = fs::read_to_string(file_path)?;
    
    // Extract name from filename or use custom name
    let name = if let Some(custom) = custom_name {
        custom
    } else if let Some(filename) = file_path.file_stem().and_then(|n| n.to_str()) {
        // Convert filename back to readable format
        filename.replace('_', " ").to_string()
    } else {
        file_query.to_string()
    };
    
    println!("📖 Loading snippet from: {}", file_path.display());
    
    Ok((content, name))
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