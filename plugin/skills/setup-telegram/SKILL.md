---
name: setup-telegram
description: "Configure Telegram notifications for claude-notify. Requires a bot token and chat ID."
---

# Setup Telegram Notifications

Run `claude-notify setup telegram <BOT_TOKEN> <CHAT_ID>` to configure Telegram as a notification backend.

## How to get credentials

1. Message [@BotFather](https://t.me/BotFather) on Telegram, send `/newbot`, follow prompts to get a **bot token**
2. Message [@userinfobot](https://t.me/userinfobot) to get your **chat ID**

## Usage

The user provides a bot token and chat ID as arguments.

Example: `claude-notify setup telegram 123456:ABC-DEF 987654321`

Run the command with the user's credentials, then show the output.
