---
name: setup-teams
description: "Configure Microsoft Teams notifications for claude-notify. Requires a Workflows webhook URL."
---

# Setup Microsoft Teams Notifications

Run `claude-notify setup teams <WEBHOOK_URL>` to configure Teams as a notification backend.

## How to get a webhook URL

1. In Teams, create an Incoming Webhook via Workflows (Power Automate)
2. Note: legacy Office 365 connectors are deprecated
3. Copy the webhook URL

## Usage

The user provides a Teams webhook URL as the argument.

Example: `claude-notify setup teams https://xxx.webhook.office.com/webhookb2/...`

Run the command with the user's webhook URL, then show the output.
