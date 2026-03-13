---
name: dry-run
description: "Test notification formatting by running sample hook events through --dry-run for each event type (permission_prompt, idle_prompt, elicitation_dialog, stop, task_completed)"
---

# Dry Run Notification Testing

Run each of the following sample hook events through `cargo run -- --dry-run` and display the formatted output. Pipe the JSON to stdin.

## Test Cases

### 1. Permission prompt
```bash
echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"Notification","notification_type":"permission_prompt","tool_name":"Bash","tool_input":{"command":"npm install express"}}' | cargo run -- --dry-run
```

### 2. Idle prompt
```bash
echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"Notification","notification_type":"idle_prompt"}' | cargo run -- --dry-run
```

### 3. Elicitation dialog
```bash
echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"Notification","notification_type":"elicitation_dialog","message":"Which database should I use?"}' | cargo run -- --dry-run
```

### 4. Stop (response complete)
```bash
echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"Stop","last_assistant_message":"I fixed the bug in the login handler."}' | cargo run -- --dry-run
```

### 5. Task completed
```bash
echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"TaskCompleted","task_subject":"Fix auth bug","teammate_name":"implementer","task_description":"Fix the authentication timeout issue"}' | cargo run -- --dry-run
```

## What to Check

- All 5 event types produce formatted output without errors
- Emoji icons are correct for each event type
- Session friendly name is consistent for the same session_id
- Project name is extracted from cwd path
- HTML tags (`<b>`, etc.) are present in output
