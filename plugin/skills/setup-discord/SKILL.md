---
name: setup-discord
description: "Configure Discord notifications for claude-notify. Requires a webhook URL."
---

# Setup Discord Notifications

Run `claude-notify setup discord <WEBHOOK_URL>` to configure Discord as a notification backend.

## How to get a webhook URL

1. In your Discord server, go to a channel's settings
2. Navigate to Integrations -> Webhooks
3. Create a new webhook and copy the URL

## Usage

The user provides a Discord webhook URL as the argument.

Example: `claude-notify setup discord https://discord.com/api/webhooks/123/abc`

Run the command with the user's webhook URL, then show the output.
