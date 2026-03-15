---
name: setup-ntfy
description: "Configure ntfy push notifications for claude-notify. Requires a topic URL."
---

# Setup ntfy Notifications

Run `claude-notify setup ntfy <TOPIC_URL>` to configure ntfy as a notification backend.

## How to get a topic URL

1. Pick a topic name at [ntfy.sh](https://ntfy.sh) (or use your own ntfy server)
2. Subscribe to the topic on your phone via the ntfy app
3. Your topic URL is `https://ntfy.sh/your-topic-name`

## Usage

The user provides an ntfy topic URL as the argument.

Example: `claude-notify setup ntfy https://ntfy.sh/my-claude-topic`

Run the command with the user's topic URL, then show the output.
