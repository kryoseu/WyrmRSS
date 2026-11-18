---
title: Runtime settings
description: Settings you can change from the Settings page without restarting.
sidebar:
  order: 2
---

The following settings are stored in the database and can be changed at runtime via the Settings page without restarting the server.

| Setting | Default | Description |
| --- | --- | --- |
| Page size | `100` | Feed entries returned per page |
| Feed poll interval | `900` | Seconds between automatic feed polls |
| HTTP timeout | `30` | Response timeout in seconds |
| HTTP connect timeout | `10` | Connection timeout in seconds |
| HTTP retries | `3` | Max retries for failed feed fetches |
| HTTP user agent | `wyrm-rss/<version>` | User agent sent with feed requests |
| Mark as read | `On open` | When posts are marked read: `On open`, `Manually`, or `Disabled` (no read tracking) |
| Mark as read delay | `0` | Seconds a post must stay open before it's marked read (`On open` mode only); `0` marks it immediately |
| Delete read posts after | _(never)_ | Days before read posts are deleted; Read Later and archived posts are kept |
| Delete unread posts after | _(never)_ | Days before unread posts are deleted; Read Later and archived posts are kept |
