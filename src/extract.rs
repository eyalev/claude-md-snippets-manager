use anyhow::{Result, Context};
use std::fs;
use std::path::Path;
use std::process::Command;
use tokio::fs as async_fs;

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
    let extracted_content = extract_with_claude_code(&query, &claude_md_path).await?;
    
    // Generate filename from query (sanitized)
    let filename = sanitize_filename(&query) + ".md";
    let output_path = local_snippets_dir.join(&filename);
    
    // Write extracted content to file
    async_fs::write(&output_path, extracted_content)
        .await
        .context("Failed to write extracted content to file")?;
    
    println!("✓ Extracted snippet saved to: {}", output_path.display());
    
    Ok(())
}

async fn extract_with_claude_code(query: &str, claude_md_path: &Path) -> Result<String> {
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
    
    // Add metadata header
    let content = format!(
        "# {}\n\n<!-- Extracted from ~/.claude/CLAUDE.md using claude-md-snippets -->\n<!-- Query: {} -->\n<!-- Date: {} -->\n\n{}",
        query,
        query,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        extracted.trim()
    );
    
    Ok(content)
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