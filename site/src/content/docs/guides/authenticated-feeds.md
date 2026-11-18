---
title: Feeds behind authentication
description: Per-host basic auth or custom header credentials for protected feeds.
---

For feeds behind HTTP basic auth (e.g. `.htaccess`) or an API key header, add
a credential in the **Credentials** section of **Settings**. Credentials are per host, not per
feed, so one entry covers every feed on that host. They're sent with feed
polls, feed discovery and icon fetches, but not webhooks.

| Kind | Sent as |
| --- | --- |
| Basic auth | `Authorization: Basic …` from a username and password |
| Custom header | Any header and value, e.g. `X-Api-Key: …` or `Authorization: Bearer …` |

Credentials match on host **and** port. A bare host like `feeds.example.com`
means `https` on port 443. For an `http://` feed enter
`http://feeds.example.com`, and for a custom port add it:
`feeds.example.com:8443`.

:::caution
Secrets are never returned by the API once saved; they can be replaced but
not read back. They are stored unencrypted in the database.
:::
