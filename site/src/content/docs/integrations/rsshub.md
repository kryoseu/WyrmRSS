---
title: RSSHub
description: Follow sites without RSS through your RSSHub instance.
---

[RSSHub](https://docs.rsshub.app) generates feeds for sites without RSS. Set
your instance URL under **RSSHub** in the **Integrations** section of **Settings** (e.g.
`http://localhost:1200`).

Add RSSHub feeds as `rsshub://<route>`, e.g. `rsshub://hackernews`. You can
also paste a full URL from your instance, such as
`http://localhost:1200/hackernews` (the kind RSSHub Radar gives you). Wyrm
saves it as `rsshub://hackernews`, so it keeps working if you switch instances.

Turning the integration off pauses RSSHub feeds until you turn it back on.
