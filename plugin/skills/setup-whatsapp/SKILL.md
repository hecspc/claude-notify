---
name: setup-whatsapp
description: "Configure WhatsApp notifications for claude-notify via Meta Cloud API. Requires phone number ID, access token, and recipient number."
---

# Setup WhatsApp Notifications

Run `claude-notify setup whatsapp <PHONE_NUMBER_ID> <ACCESS_TOKEN> <RECIPIENT>` to configure WhatsApp as a notification backend.

## How to get credentials

1. Create a [Meta Developer](https://developers.facebook.com/) account
2. Create an app and add WhatsApp as a product
3. In the WhatsApp section, get your **Phone Number ID** and generate a **permanent access token**
4. The **recipient** is the phone number to receive notifications, in international format without `+` (e.g. `14155551234`)

## Usage

The user provides three arguments: phone number ID, access token, and recipient phone number.

Example: `claude-notify setup whatsapp 123456789 EAAxxxxxxx 14155551234`

Run the command with the user's credentials, then show the output.
