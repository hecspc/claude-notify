---
name: setup-openclaw
description: "Configure OpenClaw notifications for claude-notify via Gateway API. Routes notifications through OpenClaw to any connected channel (WhatsApp, Telegram, Discord, etc.)."
---

# Setup OpenClaw Notifications

Run `claude-notify setup openclaw <GATEWAY_URL> <TOKEN> <TARGET>` to configure OpenClaw as a notification backend.

## How to get credentials

1. Install and run [OpenClaw](https://openclaw.ai/) Gateway
2. Get your Gateway URL (e.g. `http://localhost:3000`)
3. Get your Bearer token from the Gateway auth configuration
4. Determine your target (phone number, user ID, or channel identifier)

## Usage

Required arguments: gateway URL, token, and target.

Optional: `--channel` to specify delivery channel (e.g. whatsapp, telegram, discord).

Examples:
- `claude-notify setup openclaw http://localhost:3000 my-token +15555550123`
- `claude-notify setup openclaw http://localhost:3000 my-token +15555550123 --channel whatsapp`

Run the command with the user's credentials, then show the output.
