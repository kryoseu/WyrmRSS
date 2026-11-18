---
title: Server configuration
description: Startup settings for the Wyrm server and web UI.
sidebar:
  order: 1
---

## Server

Startup settings can be configured via environment variables (prefixed `WYRM_`) or `config/wyrm.toml`.

| Variable | Default | Description |
| --- | --- | --- |
| `WYRM_BIND` | `0.0.0.0` | IP address to bind to |
| `WYRM_PORT` | `3001` | Port to listen on |
| `WYRM_DATABASE_CONNECTION` | `postgres://wyrm:wyrm@localhost/wyrm` | PostgreSQL connection URI |
| `WYRM_DATABASE_POOL_SIZE` | `30` | Max database connection pool size |
| `WYRM_PUBLIC_URL` | _(none)_ | URL you reach Wyrm on, e.g. `https://rss.example.com`; required for OIDC, whose callback is `{public_url}/api/v1/auth/oidc/callback` |
| `WYRM_API_KEY` | _(none)_ | API key accepted independently of OIDC; the API is open when neither authentication method is configured |
| `WYRM_OIDC_ISSUER_URL` | _(none)_ | Enable optional OIDC using this provider's issuer URL |
| `WYRM_OIDC_CLIENT_ID` | _(none)_ | Registered OIDC client ID |
| `WYRM_OIDC_CLIENT_SECRET` | _(none)_ | OIDC client secret; omit for public clients |
| `WYRM_OIDC_ALLOWED_SUBJECTS` | _(none)_ | Optional comma-separated `sub` allowlist; unset allows any user authenticated by the provider |

:::note
Setting `WYRM_API_KEY` on the backend is enough to restrict access. Setting it on the frontend too skips the in-app key prompt. When both API-key and OIDC settings are absent, the API remains open.
:::

OIDC is disabled by default. See [OpenID Connect (OIDC)](/configuration/oidc/) for provider registration, configuration, and session behavior. A valid `x-api-key` always works without an OIDC login when `WYRM_API_KEY` is configured, including when OIDC is enabled.

## Web UI

| Variable | Default | Description |
| --- | --- | --- |
| `WYRM_BACKEND_URL` | `http://localhost:3001` | Backend URL, must match where the backend is running |
| `WYRM_API_KEY` | _(none)_ | Bake an API key into the frontend build, skipping the key prompt entirely |
