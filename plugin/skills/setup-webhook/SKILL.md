---
name: setup-webhook
description: "Configure generic webhook notifications for claude-notify. Supports unnamed and named webhook instances."
---

# Setup Webhook Notifications

Configure a generic webhook to receive notification POSTs.

## Usage

**Unnamed webhook:** `claude-notify setup webhook <URL>`
**Named instance:** `claude-notify setup webhook <NAME> <URL>`

Examples:
- `claude-notify setup webhook https://example.com/notify`
- `claude-notify setup webhook ha-appletv http://homeassistant:8123/api/webhook/claude-notify`

The webhook receives a POST with JSON: `{"title": "...", "body": "...", "text": "..."}`

Run the command with the user's arguments, then show the output.
