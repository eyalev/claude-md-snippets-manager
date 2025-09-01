# Claude MD Snippets

A Rust CLI tool for managing and sharing CLAUDE.md snippets. Publish your snippets, discover community snippets, and install them with smart matching powered by Claude Code.

## Features

- **Publish**: Create and share CLAUDE.md snippets
- **Install**: Intelligently find and install snippets using Claude Code
- **Search**: Fuzzy search through snippets with fzf integration  
- **Sync**: GitHub integration for sharing snippets with the community

## Installation

```bash
cargo install --path .
```

## Quick Start

### Publish a snippet
```bash
claude-md-snippets publish "## Running GUI Applications

When opening GUI applications like Chrome, Firefox, etc., always use \`nohup\` and redirect output to /dev/null to run them completely detached:

\`\`\`bash
nohup google-chrome > /dev/null 2>&1 &
\`\`\`

This prevents the terminal from getting stuck on application output."
```

### Install a snippet
```bash
claude-md-snippets install "gui applications chrome"
```

### Search and browse snippets
```bash
claude-md-snippets search
```

### Sync with GitHub
```bash
# Push local snippets to repository
claude-md-snippets sync

# Pull latest snippets from repository  
claude-md-snippets pull
```

## How it Works

1. **Publishing**: Snippets are saved locally as JSON files with auto-generated names and metadata
2. **Installing**: Uses Claude Code to analyze snippet content and find the best match for your query
3. **Search**: Integrates with fzf for interactive fuzzy searching with live previews
4. **GitHub Integration**: Manages a local git repository for syncing snippets with the community

## Storage

- Local snippets: `~/.claude-snippets/`
- Installed to: `./CLAUDE.md` or `~/.claude/CLAUDE.md`

## Requirements

- Rust (for building)
- fzf (for search functionality)
- git (for sync functionality)
- Claude Code (for intelligent snippet matching)

## Examples

### Custom snippet name
```bash
claude-md-snippets publish "content here" --name "my-custom-name"
```

### Install with fallback matching
If Claude Code is not available, the tool falls back to fuzzy text matching.

## Contributing

This tool is designed to work with any GitHub repository. To set up your own snippet repository:

1. Create a GitHub repository
2. Configure the remote: `cd ~/.claude-snippets && git remote add origin <your-repo-url>`  
3. Push your snippets: `claude-md-snippets sync`

## License

MIT License