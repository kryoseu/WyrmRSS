# Build from source

Steps on how to build Wyrm from source.

## Prerequisites

- Rust (stable toolchain)
- PostgreSQL
- Node.js
- pnpm

## Build from source

First clone the repository:

```bash
git clone git@github.com:kryoseu/WyrmRSS.git
```

Run the backend:

```bash
cargo run -p server
```

The server reads configuration from `config/wyrm.toml` in the working directory. Copy and edit the example:

```bash
cp config/wyrm-example.toml config/wyrm.toml
```

Then install dependencies and run the frontend:

```bash
export WYRM_BACKEND_URL="http://localhost:3001"

cd web
pnpm install
pnpm build
pnpm preview
```

## Configuration

Backend configuration via  `config/wyrm.toml`:

```toml
port = 3001
# api_key = "secret123!"
database_connection = "postgres://wyrm:wyrm@localhost/wyrm"
database_pool_size = 25      # default 30
```

Settings can also be overridden with environment variables prefixed `WYRM_`, e.g. `WYRM_PORT=8080`.

Open Wyrm at <http://localhost:3000>.
