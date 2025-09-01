---
id: 769eb3ad-eb38-4c36-b3d1-6c6d76ad1227
name: Running GUI Applications
created_at: 2025-09-01T13:00:50.203098606+00:00
description: Extracted from ~/.claude/CLAUDE.md
source: extract
query: Running GUI Applications
---

# Running GUI Applications

When opening GUI applications like Chrome, Firefox, etc., always use `nohup` and redirect output to /dev/null to run them completely detached:

```bash
nohup google-chrome > /dev/null 2>&1 &
```

This prevents the terminal from getting stuck on application output and allows the process to run independently.