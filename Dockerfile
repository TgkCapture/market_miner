# --- Dashboard -----------------------------------------------------------
FROM node:22-alpine AS web
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# --- Server --------------------------------------------------------------
FROM rust:1.89-slim-bookworm AS server
WORKDIR /app
# Build dependencies first so they're cached between source changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && touch src/lib.rs \
    && cargo build --release --locked && rm -rf src
COPY migrations ./migrations
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

# --- Runtime -------------------------------------------------------------
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=server /app/target/release/market_miner /usr/local/bin/market_miner
COPY --from=web /web/dist ./web/dist
USER app
ENV BIND_ADDR=0.0.0.0:8080 STATIC_DIR=/app/web/dist
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s \
  CMD curl -fsS http://localhost:8080/api/health || exit 1
CMD ["market_miner"]
