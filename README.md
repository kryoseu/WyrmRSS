<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="images/l-o-light.svg" width="65%">
    <img src="images/l-o-dark.svg" width="65%">
  </picture>
</p>

<p align="center">
  <i>A self-hosted RSS reader and aggregator, also available as a desktop app</i>.
</p>

<p align="center">
  <img src="images/wyrm.png" width="80%" />
</p>

## Features

- Subscribe to RSS and Atom feeds
- Feed discovery — paste a blog homepage or YouTube channel URL and Wyrm finds the feed for you
- YouTube channel support — subscribe to any channel and watch videos inline without leaving the reader
- Browse posts per feed or across all feeds
- Per-feed display modes — River (home feed), Feed only (posts only shown on Feed directly), or Radar (quiet, visit when nothing else going on)
- Group feeds into folders — collapsible sidebar sections with unread counts, preserved across OPML import/export
- Filters per feed — exclude entries matching a pattern, scoped to URL, title or content
- Save posts to Read Later
- Archive posts
- Webhooks - get notified on new posts (Discord, Slack and custom)
- OPML import and export
- Background feed polling with configurable interval and retry logic
- Full-text search on post titles and description
- Hot-reloaded settings (poll interval, HTTP timeouts, page size, reading mode) configurable via the UI
- Easy to install

## Installation

Docker compose and Kubernetes manifests are provided for an easy self-hosted
install. A native desktop app is also available, with no server or database
to set up. You may also build it from source.

### Desktop app

See [desktop.md](docs/installation/desktop.md) for macOS, Windows and Linux downloads.

### With Docker

See [docker.md](docs/installation/docker.md) for Docker Compose setup.

### With Kubernetes

See [kubernetes.yaml](docs/templates/kubernetes.yaml) for Kubernetes manifests.

### From source

See [source.md](docs/installation/source.md) to build from the source code.

## Configuration

### Startup configuration

Startup settings can be configured via environment variables (prefixed `WYRM_`) or `config/wyrm.toml`.

| Variable | Default | Description |
| --- | --- | --- |
| `WYRM_BIND` | `0.0.0.0` | IP address to bind to |
| `WYRM_PORT` | `3001` | Port to listen on |
| `WYRM_DATABASE_CONNECTION` | `postgres://wyrm:wyrm@localhost/wyrm` | PostgreSQL connection URI |
| `WYRM_DATABASE_POOL_SIZE` | `30` | Max database connection pool size |
| `WYRM_API_KEY` | _(none)_ | API key required by clients; unset means the API is open and any key is accepted |

> [!NOTE]
> Setting `WYRM_API_KEY` on the backend is enough to restrict access. Setting it on the frontend too skips the in-app key prompt. If left unset on the backend, the API is open and any key entered in the prompt will be accepted.

### Runtime settings

The following settings are stored in the database and can be changed at runtime via the Settings page without restarting the server.

| Setting | Default | Description |
| --- | --- | --- |
| Page size | `100` | Feed entries returned per page |
| Feed poll interval | `900` | Seconds between automatic feed polls |
| HTTP timeout | `30` | Response timeout in seconds |
| HTTP connect timeout | `10` | Connection timeout in seconds |
| HTTP retries | `3` | Max retries for failed feed fetches |
| HTTP user agent | `wyrm-rss/<version>` | User agent sent with feed requests |

### Frontend

| Variable | Default | Description |
| --- | --- | --- |
| `WYRM_BACKEND_URL` | `http://localhost:3001` | Backend URL, must match where the backend is running |
| `WYRM_API_KEY` | _(none)_ | Bake an API key into the frontend build, skipping the key prompt entirely |

## YouTube

Wyrm treats YouTube channels as regular RSS feeds. Add a channel using its feed URL:

```
https://www.youtube.com/feeds/videos.xml?channel_id=CHANNEL_ID
```

The reader detects YouTube links and renders an embedded player in place of the article body.

### Filtering content

Use filters to exclude entries you don't want from a feed. Each filter is a
case-insensitive substring pattern, optionally scoped to a field with a
`url:`, `title:` or `content:` prefix; an unprefixed pattern matches any of
them. `content:` matches both the entry's summary and body.

Example, for YouTube you may want to exclude:

| Filter | Excludes |
| -------- | ---------- |
| `url:/shorts` | YouTube Shorts |
| `url:/live` | Live streams |
| `title:trailer` | Entries with "trailer" in the title |

Filters are per-feed, so `url:/shorts` drops any entry from that feed whose
URL contains `/shorts`.

## Stack

| Layer    | Technology                              |
|----------|-----------------------------------------|
| Backend  | Rust, Actix-web, Diesel, Tokio          |
| Database | PostgreSQL (self-hosted) / SQLite (desktop app) |
| Frontend | React, Vite, React Router v7, TanStack Query |
| Desktop  | Tauri                                   |

## Star History

<a href="https://www.star-history.com/?repos=kryoseu%2FWyrmRSS&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=kryoseu/WyrmRSS&type=date&theme=dark&legend=top-left&sealed_token=HFZ6f_QY-lCQIXL3wU7i9RhU_PAy8mbPXIQSh1Oigg-XyYosIvbOstQHU69deVQbPPUWzEp_v779rvHomnLzCQqo2nN7plx0aCJKIgEHqagjA9dnAUkP6nPIkn5cLlhCanOTylZMIXZ2-H-Wyyfk3FEQ_FMwG4ZjCwUUrqAIXqwRQmwd4keJjb8j6bu1" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=kryoseu/WyrmRSS&type=date&legend=top-left&sealed_token=HFZ6f_QY-lCQIXL3wU7i9RhU_PAy8mbPXIQSh1Oigg-XyYosIvbOstQHU69deVQbPPUWzEp_v779rvHomnLzCQqo2nN7plx0aCJKIgEHqagjA9dnAUkP6nPIkn5cLlhCanOTylZMIXZ2-H-Wyyfk3FEQ_FMwG4ZjCwUUrqAIXqwRQmwd4keJjb8j6bu1" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=kryoseu/WyrmRSS&type=date&legend=top-left&sealed_token=HFZ6f_QY-lCQIXL3wU7i9RhU_PAy8mbPXIQSh1Oigg-XyYosIvbOstQHU69deVQbPPUWzEp_v779rvHomnLzCQqo2nN7plx0aCJKIgEHqagjA9dnAUkP6nPIkn5cLlhCanOTylZMIXZ2-H-Wyyfk3FEQ_FMwG4ZjCwUUrqAIXqwRQmwd4keJjb8j6bu1" />
 </picture>
</a>

## License

GPL v3 — see [LICENSE](LICENSE).

## Credits

Logo by [Pedro Niels](https://www.instagram.com/opedroniels/).
