<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="images/1-dark.png" width="55%">
    <img src="images/1-light.png width="55%">
  </picture>
</p>

<p align="center">
  A self-hosted RSS reader and aggregator.
</p>

<p align="center">
  <img src="images/2.png" />
</p>

## Features

- Subscribe to RSS and Atom feeds
- YouTube channel support — subscribe to any channel's RSS feed and watch videos inline without leaving the reader
- Browse posts per feed or across all feeds
- URL filters per feed — exclude entries whose URL contains a given pattern
- Mark posts as favorite
- Background feed polling with configurable TTL and retry logic
- Full-text search on post titles
  
  
## Installation

Docker compose and Kubernetes manifests are provided for an easy install.
You may also build it from source.

### With Docker

See [docker.md](docs/installation/docker.md) for Docker Compose setup.

### With Kubernetes

See [kubernetes.yaml](docs/templates/kubernetes.yaml) for Kubernetes manifests.

### From source

See [source.md](docs/installation/source.md) to build from the source code.

## Configuration

All backend settings can be configured via environment variables (prefixed `WYRM_`) or `config/wyrm.toml`.

### Backend

| Variable | Default | Description |
|---|---|---|
| `WYRM_BIND` | `0.0.0.0` | IP address to bind to |
| `WYRM_PORT` | `3001` | Port to listen on |
| `WYRM_DATABASE_CONNECTION` | `postgres://wyrm:wyrm@localhost/wyrm` | PostgreSQL connection URI |
| `WYRM_DATABASE_POOL_SIZE` | `30` | Max database connection pool size |
| `WYRM_HTTP_TIMEOUT` | `30` | HTTP response timeout in seconds |
| `WYRM_HTTP_CONNECT_TIMEOUT` | `30` | HTTP connect timeout in seconds |
| `WYRM_HTTP_RETRIES` | `3` | Max retries for feed fetches |
| `WYRM_HTTP_USER_AGENT` | `wyrm/<version>` | User agent sent with feed requests |
| `WYRM_FEED_PAGE_SIZE` | `100` | Feed entries returned per page |
| `RUST_LOG` | `info` | Log level filter (e.g. `debug`, `wyrm=trace`) |

### Frontend

| Variable | Default | Description |
|---|---|---|
| `WYRM_BACKEND_URL` | `http://localhost:3001` | Backend URL used by the Vite preview proxy |

## YouTube

Wyrm treats YouTube channels as regular RSS feeds. Add a channel using its feed URL:

```
https://www.youtube.com/feeds/videos.xml?channel_id=CHANNEL_ID
```

The reader detects YouTube links and renders an embedded player in place of the article body.

### Filtering content

Use URL filters to exclude certain types you don't want from a feed. Example, for YouTube you may want to exclude:

| Filter | Excludes |
|--------|----------|
| `/shorts` | YouTube Shorts |
| `/live` | Live streams |

Filters are per-feed and substring-matched against each entry's URL, so `/shorts` will drop any entry from that feed whose URL contains `/shorts`.

## Stack

| Layer    | Technology                              |
|----------|-----------------------------------------|
| Backend  | Rust, Actix-web, Diesel, Tokio          |
| Database | PostgreSQL                              |
| Frontend | React, Vite, React Router v7, TanStack Query |

## License

GPL v3 — see [LICENSE](LICENSE).
