FROM rust:1-bookworm AS chef
RUN cargo install --locked cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends libpq5 ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && groupadd -r wyrm && useradd -r -g wyrm wyrm
COPY --from=builder /app/target/release/server /usr/local/bin/server
WORKDIR /app

USER wyrm

EXPOSE 3001

ENTRYPOINT ["server"]
