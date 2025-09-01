use anyhow::{Result, Context};
use std::fs;
use std::path::Path;
use std::process::Command;
use tokio::fs as async_fs;
use uuid::Uuid;

pub async fn extract_snippet(query: String) -> Result<()> {
    println!("Extracting information about: {}", query);
    
    // Check if ~/.claude/CLAUDE.md exists
    let home_dir = dirs::home_dir()
        .context("Could not find home directory")?;
    let claude_md_path = home_dir.join(".claude/CLAUDE.md");
    
    if !claude_md_path.exists() {
        anyhow::bail!("~/.claude/CLAUDE.md not found");
    }
    
    // Create local .claude.local/snippets directory
    let local_snippets_dir = Path::new("./.claude.local/snippets");
    fs::create_dir_all(local_snippets_dir)
        .context("Failed to create ./.claude.local/snippets directory")?;
    
    // Use Claude Code to extract relevant information
    let (extracted_content, snippet_id) = extract_with_claude_code(&query, &claude_md_path).await?;
    
    // Generate filename from query (sanitized) with ID
    let filename = format!("{}-{}.md", sanitize_filename(&query), &snippet_id[..8]);
    let output_path = local_snippets_dir.join(&filename);
    
    // Write extracted content to file
    async_fs::write(&output_path, extracted_content)
        .await
        .context("Failed to write extracted content to file")?;
    
    println!("✓ Extracted snippet saved to: {}", output_path.display());
    
    Ok(())
}

async fn extract_with_claude_code(query: &str, claude_md_path: &Path) -> Result<(String, String)> {
    println!("Using Claude Code to extract relevant information...");
    
    // Prepare the prompt for Claude Code
    let prompt = format!(
        "Please extract all relevant information about '{}' from the CLAUDE.md file. \
        Include any related sections, instructions, code examples, or configuration details. \
        Format the output as a clean markdown snippet that can be used independently.",
        query
    );
    
    // Run Claude Code with the prompt
    let output = Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--print")
        .arg(&format!("Read the file {} and {}", claude_md_path.display(), prompt))
        .output()
        .context("Failed to execute Claude Code CLI")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Claude Code failed: {}", stderr);
    }
    
    let extracted = String::from_utf8_lossy(&output.stdout);
    
    // Create content with YAML frontmatter
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();
    
    let frontmatter = format!(
        "---\nid: {}\nname: {}\ncreated_at: {}\ndescription: Extracted from ~/.claude/CLAUDE.md\nsource: extract\nquery: {}\n---\n\n",
        id,
        query,
        timestamp,
        query
    );
    
    let content = format!("{}{}", frontmatter, extracted.trim());
    
    Ok((content, id))
}

fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .replace(' ', "_")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Running GUI Applications"), "running_gui_applications");
        assert_eq!(sanitize_filename("Test/Path\\Name"), "test_path_name");
        assert_eq!(sanitize_filename("Special-Characters!@#"), "special-characters___");
    }
}