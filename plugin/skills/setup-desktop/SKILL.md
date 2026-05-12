---
name: setup-desktop
description: "Configure native desktop notifications for claude-notify. Zero-config — no credentials needed."
---

# Setup Desktop Notifications

Run `claude-notify setup desktop` to enable native OS notifications.

No credentials needed. Uses `osascript` on macOS, `notify-send` on Linux, and PowerShell toast notifications on Windows.

## macOS click behavior (optional)

By default, clicking a macOS notification opens Script Editor. To make a click open the user's terminal (or any app) instead, install `terminal-notifier` and pass `--activate` or `--execute`:

- `brew install terminal-notifier`
- `claude-notify setup desktop --activate <BUNDLE_ID>` — click opens that app
- `claude-notify setup desktop --execute '<COMMAND>'` — click runs the command (overrides `--activate`)
- `claude-notify setup desktop --app-icon <PATH>` — custom notification icon (.png/.jpg/.icns)

Common bundle ids: `com.apple.Terminal`, `com.googlecode.iterm2`, `com.mitchellh.ghostty`, `net.kovidgoyal.kitty`, `com.github.wez.wezterm`, `org.alacritty`, `dev.warp.Warp-Stable`, `com.microsoft.VSCode`.

If the user wants a click target, ask which terminal they use, map it to the bundle id, then run with `--activate`. Otherwise run plain `setup desktop`.

Run the command, then show the output.
