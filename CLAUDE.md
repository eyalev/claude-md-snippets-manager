

# Running GUI Applications

When opening GUI applications like Chrome, Firefox, etc., always use `nohup` and redirect output to /dev/null to run them completely detached:

```bash
nohup google-chrome > /dev/null 2>&1 &
```

This prevents the terminal from getting stuck on application output and allows the process to run independently.