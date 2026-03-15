---
name: setup-email
description: "Configure email notifications for claude-notify via SMTP. Requires sender, recipient, SMTP host, username, and password."
---

# Setup Email Notifications

Run `claude-notify setup email <FROM> <TO> <SMTP_HOST> <USERNAME> <PASSWORD>` to configure email notifications.

## Usage

The user provides SMTP credentials. Uses STARTTLS on port 587 by default.

Example: `claude-notify setup email sender@example.com recipient@example.com smtp.example.com user password`

For Gmail, use an [App Password](https://support.google.com/accounts/answer/185833) with `smtp.gmail.com`.

Run the command with the user's credentials, then show the output.
