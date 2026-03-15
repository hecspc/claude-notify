---
name: setup-slack
description: "Configure Slack notifications for claude-notify. Requires an Incoming Webhook URL."
---

# Setup Slack Notifications

Run `claude-notify setup slack <WEBHOOK_URL>` to configure Slack as a notification backend.

## How to get a webhook URL

1. Go to [Slack API: Incoming Webhooks](https://api.slack.com/messaging/webhooks)
2. Create a new app or use an existing one
3. Enable Incoming Webhooks and add one to your desired channel
4. Copy the webhook URL

## Usage

The user provides a Slack webhook URL as the argument.

Example: `claude-notify setup slack https://hooks.slack.com/services/T.../B.../xxx`

Run the command with the user's webhook URL, then show the output.
