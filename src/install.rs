use anyhow::Result;
use std::fs;
use std::process::Command;
use std::io::Write;
use crate::publish::{Snippet, get_snippets_dir};

pub async fn install_snippet(query: String, force_local: bool, force_user: bool) -> Result<()> {
    // Load all available snippets
    let snippets = load_snippets()?;
    
    if snippets.is_empty() {
        println!("❌ No snippets found. Try publishing some first!");
        return Ok(());
    }

    println!("🔍 Finding best match for: '{}'", query);
    
    // Use Claude Code to find the best matching snippet
    let best_match = find_best_match(&snippets, &query).await?;
    
    if let Some(snippet) = best_match {
        println!("✅ Found matching snippet: '{}'", snippet.name);
        println!("📋 Content preview:");
        println!("{}", preview_content(&snippet.content));
        
        // Confirm installation
        print!("Install this snippet to CLAUDE.md? [Y/n]: ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        if input.is_empty() || input == "y" || input == "yes" {
            install_to_claude_md(&snippet, force_local, force_user).await?;
            println!("✅ Snippet installed successfully!");
        } else {
            println!("❌ Installation cancelled");
        }
    } else {
        println!("❌ No suitable snippet found for query: '{}'", query);
        println!("💡 Available snippets:");
        for snippet in &snippets {
            println!("  - {}", snippet.name);
        }
    }
    
    Ok(())
}

async fn find_best_match(snippets: &[Snippet], query: &str) -> Result<Option<Snippet>> {
    // Create a temporary file with snippet information for Claude Code to analyze
    let temp_dir = std::env::temp_dir();
    let snippets_file = temp_dir.join("claude_snippets_analysis.json");
    
    let analysis_data = serde_json::json!({
        "query": query,
        "snippets": snippets.iter().map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "content_preview": preview_content(&s.content),
            "full_content": s.content
        })).collect::<Vec<_>>()
    });
    
    fs::write(&snippets_file, serde_json::to_string_pretty(&analysis_data)?)?;
    
    // Use Claude Code to analyze and find the best match
    let claude_prompt = format!(
        "Analyze the following snippets and find the best match for the query: '{}'\n\
        Return only the ID of the best matching snippet, or 'NONE' if no good match exists.\n\
        Consider semantic similarity, keywords, and practical relevance.\n\
        File: {}",
        query,
        snippets_file.display()
    );
    
    // Try to run Claude Code
    let output = Command::new("claude")
        .args(&["--dangerously-skip-permissions", "--non-interactive"])
        .arg(&claude_prompt)
        .output();
    
    match output {
        Ok(result) => {
            let response = String::from_utf8_lossy(&result.stdout).trim().to_string();
            
            if response == "NONE" {
                return Ok(None);
            }
            
            // Find the snippet with the matching ID
            for snippet in snippets {
                if snippet.id.starts_with(&response) || response.contains(&snippet.id) {
                    return Ok(Some(snippet.clone()));
                }
            }
            
            // Fallback: simple text matching
            fuzzy_match(snippets, query)
        }
        Err(_) => {
            println!("⚠️  Claude Code not available, using fuzzy matching...");
            fuzzy_match(snippets, query)
        }
    }
}

fn fuzzy_match(snippets: &[Snippet], query: &str) -> Result<Option<Snippet>> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    
    let mut scored_snippets: Vec<(usize, &Snippet)> = Vec::new();
    
    for snippet in snippets {
        let content_lower = format!("{} {}", snippet.name, snippet.content).to_lowercase();
        let mut score = 0;
        
        // Score based on word matches
        for word in &query_words {
            if content_lower.contains(word) {
                score += word.len();
            }
        }
        
        // Bonus for name matches
        if snippet.name.to_lowercase().contains(&query_lower) {
            score += 50;
        }
        
        if score > 0 {
            scored_snippets.push((score, snippet));
        }
    }
    
    // Sort by score (highest first)
    scored_snippets.sort_by(|a, b| b.0.cmp(&a.0));
    
    Ok(scored_snippets.first().map(|(_, snippet)| (*snippet).clone()))
}

pub async fn install_to_claude_md(snippet: &Snippet, force_local: bool, force_user: bool) -> Result<()> {
    let claude_md_path = get_claude_md_path(force_local, force_user)?;
    
    // Read existing CLAUDE.md content
    let existing_content = if claude_md_path.exists() {
        fs::read_to_string(&claude_md_path)?
    } else {
        String::new()
    };
    
    // Check if snippet content already starts with a header
    let snippet_content = snippet.content.trim();
    let already_has_header = snippet_content.lines().next()
        .map(|line| line.trim().starts_with('#'))
        .unwrap_or(false);
    
    let new_content = if already_has_header {
        // Just add the content with a separator comment
        format!("{}\n\n{}", existing_content, snippet_content)
    } else {
        // Add header for content without one
        let snippet_header = format!("\n\n# {} (installed snippet)\n\n", snippet.name);
        format!("{}{}{}", existing_content, snippet_header, snippet_content)
    };
    
    // Write back to CLAUDE.md
    fs::write(&claude_md_path, new_content)?;
    
    println!("📝 Added to: {}", claude_md_path.display());
    
    Ok(())
}

fn get_claude_md_path(force_local: bool, force_user: bool) -> Result<std::path::PathBuf> {
    if force_local {
        // Force local installation
        let current_dir = std::env::current_dir()?;
        return Ok(current_dir.join("CLAUDE.md"));
    }
    
    if force_user {
        // Force user installation
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let claude_dir = home.join(".claude");
        fs::create_dir_all(&claude_dir)?;
        return Ok(claude_dir.join("CLAUDE.md"));
    }
    
    // Use config default
    let config = crate::config::Config::load()?;
    let default_location = config.get_default_install_location();
    
    match default_location {
        "local" => {
            let current_dir = std::env::current_dir()?;
            Ok(current_dir.join("CLAUDE.md"))
        }
        "user" => {
            let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
            let claude_dir = home.join(".claude");
            fs::create_dir_all(&claude_dir)?;
            Ok(claude_dir.join("CLAUDE.md"))
        }
        _ => {
            // Fallback to local
            let current_dir = std::env::current_dir()?;
            Ok(current_dir.join("CLAUDE.md"))
        }
    }
}

fn load_snippets() -> Result<Vec<Snippet>> {
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
    
    // Sort by creation date (newest first)
    snippets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    Ok(snippets)
}

fn preview_content(content: &str) -> String {
    let lines: Vec<&str> = content.lines().take(5).collect();
    let preview = lines.join("\n");
    
    if content.lines().count() > 5 {
        format!("{}\n... (truncated)", preview)
    } else {
        preview
    }
}