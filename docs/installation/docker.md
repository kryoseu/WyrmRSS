# Installation with Docker

Install Wyrm using Docker compose.

### Prerequisites

- Docker
- docker-compose

## Compose

Everything you need to run Wyrm is there, including PostgreSQL itself. You can also use your own database.

```yaml
services:
  wyrm-db:
    image: postgres:18
    container_name: wyrm-db
    environment:
      POSTGRES_USER: wyrm
      POSTGRES_PASSWORD: wyrm
      POSTGRES_DB: wyrm
    volumes:
      - wyrm_data:/var/lib/postgresql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U wyrm -d wyrm"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  wyrm-server:
    image: ghcr.io/kryoseu/wyrm-server:latest
    container_name: wyrm-rss
    environment:
      # all env vars are optional, these are default values
      WYRM_PORT: 3001
      WYRM_DATABASE_CONNECTION: "postgres://wyrm:wyrm@wyrm-db/wyrm"
      WYRM_DATABASE_POOL_SIZE: 30
      # WYRM_API_KEY: "changeme"  # set to protect the API
    ports:
      - 3001:3001
    volumes:
      - /path/to/config:/app/config
    depends_on:
      wyrm-db:
        condition: service_healthy
    restart: unless-stopped

  wyrm-ui:
    image: ghcr.io/kryoseu/wyrm-ui:latest
    container_name: wyrm-ui
    environment:
      WYRM_WEB_PORT: 3000
      WYRM_BACKEND_URL: "http://wyrm-server:3001"
      # WYRM_API_KEY: "changeme"  # must match the server's WYRM_API_KEY
    ports:
      - 3000:3000
    restart: unless-stopped
volumes:
  wyrm_data:
```

Settings can also be provided via a config file `/app/config/wyrm.toml`.

Run it:

```
docker compose -f docker-compose.yaml up
```
