use anyhow::Result;
use std::fs;
use std::process::{Command, Stdio};
use std::io::Write;
use crate::publish::{Snippet, get_snippets_dir};

pub async fn search_snippets() -> Result<()> {
    // Load all available snippets
    let snippets = load_snippets()?;
    
    if snippets.is_empty() {
        println!("❌ No snippets found. Try publishing some first!");
        return Ok(());
    }

    // Check if fzf is available
    if !is_fzf_available() {
        println!("❌ fzf is not installed. Please install it first:");
        println!("   Ubuntu/Debian: sudo apt install fzf");
        println!("   macOS: brew install fzf");
        return Ok(());
    }

    // Create formatted list for fzf
    let mut fzf_input = String::new();
    for snippet in &snippets {
        let preview = preview_content(&snippet.content, 50);
        fzf_input.push_str(&format!("{}▪{}\n", snippet.name, preview.replace('\n', " │ ")));
    }

    // Run fzf with preview
    let mut fzf_cmd = Command::new("fzf")
        .args(&[
            "--delimiter=▪",
            "--with-nth=1",
            "--preview=echo {2}",
            "--preview-window=down:3:wrap",
            "--prompt=Select snippet: ",
            "--height=50%",
            "--border",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write input to fzf
    if let Some(mut stdin) = fzf_cmd.stdin.take() {
        stdin.write_all(fzf_input.as_bytes())?;
    }

    // Get the result
    let output = fzf_cmd.wait_with_output()?;

    if output.status.success() {
        let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if !selection.is_empty() {
            // Extract the snippet name (before ▪)
            let snippet_name = selection.split('▪').next().unwrap_or("").trim();
            
            // Find the corresponding snippet
            if let Some(snippet) = snippets.iter().find(|s| s.name == snippet_name) {
                println!("\n📋 Selected snippet: {}", snippet.name);
                println!("🔍 Full content:");
                println!("{}", "─".repeat(50));
                println!("{}", snippet.content);
                println!("{}", "─".repeat(50));
                
                // Ask if user wants to install it
                print!("\nInstall this snippet to CLAUDE.md? [Y/n]: ");
                std::io::stdout().flush()?;
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                
                if input.is_empty() || input == "y" || input == "yes" {
                    crate::install::install_to_claude_md(snippet, false, false).await?;
                    println!("✅ Snippet installed successfully!");
                } else {
                    println!("❌ Installation cancelled");
                }
            }
        }
    } else {
        println!("❌ Search cancelled");
    }

    Ok(())
}

fn is_fzf_available() -> bool {
    Command::new("fzf")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn load_snippets() -> Result<Vec<Snippet>> {
    let snippets_dir = get_snippets_dir()?;
    
    if !snippets_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut snippets = Vec::new();
    
    for entry in fs::read_dir(snippets_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(snippet) = serde_json::from_str::<Snippet>(&content) {
                    snippets.push(snippet);
                }
            }
        }
    }
    
    // Sort by creation date (newest first)
    snippets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    Ok(snippets)
}

fn preview_content(content: &str, max_chars: usize) -> String {
    let content = content.replace('\n', " ");
    if content.len() > max_chars {
        format!("{}...", &content[..max_chars])
    } else {
        content
    }
}

