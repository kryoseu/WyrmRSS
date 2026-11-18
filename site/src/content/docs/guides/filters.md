---
title: Filters
description: Include or exclude feed entries by URL, title or content.
---

Use filters to exclude entries you don't want from a feed, or prefix a filter
with `+` to keep only the entries that match it. Each filter is a
case-insensitive substring pattern, optionally scoped to a field with a
`url:`, `title:` or `content:` prefix. An unprefixed pattern matches any of
them. `content:` matches both the entry's summary and body.

## Exclude filters

Example, for YouTube you may want to exclude:

| Filter | Excludes |
| -------- | ---------- |
| `url:/shorts` | YouTube Shorts |
| `url:/live` | Live streams |
| `title:trailer` | Entries with "trailer" in the title |

Filters are per-feed, so `url:/shorts` drops any entry from that feed whose
URL contains `/shorts`.

## Include filters

Include filters keep only matching entries; matching any one of them is
enough. Exclude filters still apply on top and win over includes:

| Filters | Keeps |
| -------- | ---------- |
| `+url:/shorts` | Only YouTube Shorts |
| `+title:rust`, `+title:zig` | Entries with "rust" or "zig" in the title |
| `+title:rust`, `title:jobs` | Rust entries, except job posts |

Filters apply as new entries are fetched; posts already saved aren't removed.
