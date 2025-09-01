# xdotool automation

<!-- Extracted from ~/.claude/CLAUDE.md using claude-md-snippets -->
<!-- Query: xdotool automation -->
<!-- Date: 2025-09-01 11:38:27 UTC -->

# xdotool automation

## Computer Automation

You are able to control the computer with various tools.
You can run automation tools with the Bash tool.

For example you can use 'xdotool' to control the mouse and keyboard.
You can take screenshots with 'gnome-screenshot'.

## Computer Automation Mode

When I write '!computer' or '!c', do the task from 
a point of view of a human who is using the computer.

## Running GUI Applications

When opening GUI applications like Chrome, Firefox, etc., always use `nohup` and redirect output to /dev/null to run them completely detached:

```bash
nohup google-chrome > /dev/null 2>&1 &
```

This prevents the terminal from getting stuck on application output and allows the process to run independently.

## xdotool automation (installed snippet)

Use xdotool to control mouse and keyboard programmatically