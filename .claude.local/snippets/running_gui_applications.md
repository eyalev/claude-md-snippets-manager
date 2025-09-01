# Running GUI Applications

<!-- Extracted from ~/.claude/CLAUDE.md using claude-md-snippets -->
<!-- Query: Running GUI Applications -->
<!-- Date: 2025-09-01 10:29:49 UTC -->

Here's the extracted "Running GUI Applications" section from the CLAUDE.md file:

```markdown
## Running GUI Applications

When opening GUI applications like Chrome, Firefox, etc., always use `nohup` and redirect output to /dev/null to run them completely detached:

```bash
nohup google-chrome > /dev/null 2>&1 &
```

This prevents the terminal from getting stuck on application output and allows the process to run independently.
```