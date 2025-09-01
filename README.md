# Claude MD Snippets

A powerful CLI tool for managing and sharing CLAUDE.md snippets through GitHub repositories.

## Features

- **📝 Publish**: Share snippets from your CLAUDE.md or individual files
- **📦 Install**: Install snippets with intelligent matching using Claude Code CLI
- **🔍 Search**: Interactive fuzzy finder for snippet discovery  
- **🗑️ Uninstall**: Safely remove installed snippets with ID tracking
- **🔄 Multi-repo**: Support for multiple GitHub repositories
- **⚡ Sync**: Automated GitHub synchronization
- **⚙️ Config**: Flexible configuration for default repositories and install locations

## Installation

### One-line install (Ubuntu/Linux):
```bash
curl -fsSL https://raw.githubusercontent.com/eyalev/claude-md-snippets/master/install.sh | bash
```

### Manual install:
1. Download the binary from [releases](https://github.com/eyalev/claude-md-snippets/releases)
2. Make executable: `chmod +x claude-md-snippets-linux-x64`  
3. Move to PATH: `sudo mv claude-md-snippets-linux-x64 /usr/local/bin/claude-md-snippets`

### From source:
```bash
git clone https://github.com/eyalev/claude-md-snippets.git
cd claude-md-snippets
cargo install --path .
```

## Quick Start

```bash
# Setup your first repository
claude-md-snippets setup

# Publish a snippet from current CLAUDE.md
claude-md-snippets publish "automation tools"

# Publish from a file
claude-md-snippets publish --file "my-script.sh"

# Install a snippet
claude-md-snippets install "gui applications"

# Search snippets interactively (requires fzf)
claude-md-snippets search

# Uninstall a snippet
claude-md-snippets uninstall "gui applications"
```

## Commands

### Core Commands
- `setup` - Setup GitHub repository for snippet storage
- `publish <query>` - Publish snippets from CLAUDE.md or files
- `install <query>` - Install snippets to CLAUDE.md
- `uninstall <query>` - Remove installed snippets
- `search` - Interactive snippet browser (requires fzf)

### Repository Management
- `sync` - Sync local changes with GitHub
- `pull` - Pull latest snippets from GitHub  
- `status` - Show repository status

### Configuration
- `config set-repo <name>` - Set default repository
- `config set-location <local|user>` - Set default install location
- `repo list` - List available repositories
- `repo switch <name>` - Switch to different repository

### Options
- `--local` - Install to local CLAUDE.md (current directory)
- `--user` - Install to user CLAUDE.md (~/.claude/CLAUDE.md)
- `--file <query>` - Publish from file instead of CLAUDE.md
- `--debug` - Show debug output

## How It Works

1. **Storage**: Snippets are stored as markdown files with YAML frontmatter in GitHub repositories
2. **Organization**: Multi-repository support allows organizing snippets by topic or project
3. **Installation**: Snippets are installed with HTML comment markers for safe uninstallation
4. **Intelligence**: Uses Claude Code CLI for smart snippet matching and extraction

## Directory Structure

```
~/.claude-md-snippets/
└── repos/
    ├── my-snippets/           # Local repository clone
    │   └── snippets/          # Snippet storage directory
    │       ├── snippet1.md    # Individual snippets
    │       └── snippet2.md
    └── work-snippets/         # Another repository
        └── snippets/
```

## Requirements

- Linux x86_64 (Ubuntu/Debian tested)
- Git configured with GitHub access
- GitHub CLI (`gh`) for repository operations
- Claude Code CLI (optional, for intelligent matching)
- fzf (optional, for search functionality)

## Contributing

Contributions welcome! Please open issues or submit pull requests.

## License

MIT License - see LICENSE file for details.

---

Built with ❤️ for the Claude Code community