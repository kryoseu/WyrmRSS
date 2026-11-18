---
title: YouTube
description: Subscribe to YouTube channels and watch videos in the reader.
---

Add a YouTube channel by pasting its page URL, and Wyrm finds the feed:

```
https://www.youtube.com/@HANDLE
```

Older channel links (`/channel/…`, `/c/…`, `/user/…`) work too.

The reader detects YouTube links and renders an embedded player in place of the article body.

## Hiding Shorts and live streams

Use [filters](/guides/filters/) to drop entries you don't want from a channel:

| Filter | Excludes |
| -------- | ---------- |
| `url:/shorts` | YouTube Shorts |
| `url:/live` | Live streams |
| `title:trailer` | Entries with "trailer" in the title |
