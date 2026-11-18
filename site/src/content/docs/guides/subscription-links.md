---
title: Subscription links
description: Prefill the Add Feed form from browser extensions and bookmarklets.
---

Extensions and bookmarklets can open `/subscribe` to prefill the Add Feed form
with a URL-encoded `url` and optional `title`:

```text
https://wyrm.example.com/subscribe?url=https%3A%2F%2Fexample.com%2Ffeed.xml&title=Example
```

Nothing is added until you click **Add**.

:::danger
Never put an API key in the link.
:::
