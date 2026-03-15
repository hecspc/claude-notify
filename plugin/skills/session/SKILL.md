---
name: session
description: "Toggle mute for the current Claude Code session's notifications. Mutes this session if active, unmutes if already muted."
---

# Toggle Notifications for This Session

The current session ID is: ${CLAUDE_SESSION_ID}

## Steps

1. Run `claude-notify status` to check if this session is currently muted.
2. If the session is muted, run: `claude-notify unmute ${CLAUDE_SESSION_ID}`
3. If the session is not muted, run: `claude-notify mute ${CLAUDE_SESSION_ID}`
4. Show the user the result.
