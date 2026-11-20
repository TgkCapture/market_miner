# Market Miner

Live prices and price history for the **Malawi Stock Exchange (MSE) mainboard**.

Market Miner scrapes the public MSE mainboard page on a schedule, stores every published price snapshot in
Postgres, and serves the data through a JSON API, a live update stream and a web dashboard.

- **Scraper**: reads the exchange's "stats as at" time, so re-scraping an unchanged page never stores duplicates.
- **History**: one row per security per published snapshot, ready for charts.
- **API**: latest board, top movers, per-security details and history.
- **Live**: Server-Sent Events push each new snapshot to connected clients.
- **Dashboard**: market overview, movers, sortable board and per-stock charts, in light and dark themes.

> Data comes from the MSE website and may be delayed or incomplete. This project is not affiliated with the
> Malawi Stock Exchange and is not investment advice.

## Quick start

You need [Docker](https://docs.docker.com/get-docker/).

```sh
cp .env.example .env    # optional; defaults work out of the box
docker compose up -d --build
```

Open <http://localhost:8080>. The first scrape runs at startup, so data appears within a few seconds.

Stop it with `docker compose down` (add `-v` to delete the stored data too).

## How it works

```
mse.co.mw ──scrape every 5 min──▶ parser ──▶ Postgres (securities, quotes)
                                     │
                                     └──▶ in-memory snapshot ──▶ /api/* and /api/stream ──▶ dashboard
```

| Path | What's there |
|---|---|
| [`src/scraper/parse.rs`](src/scraper/parse.rs) | HTML → snapshot. Pure functions, tested against a saved copy of the real page |
| [`src/scraper/job.rs`](src/scraper/job.rs) | The timed scrape → save → publish loop |
| [`src/db.rs`](src/db.rs) | All SQL. Schema in [`migrations/`](migrations) |
| [`src/api/`](src/api) | Axum routes and the SSE stream |
| [`src/market.rs`](src/market.rs) | Core types and calculations (change, movers) |
| [`web/`](web) | React + TypeScript dashboard (Vite) |

**About "change":** on the MSE mainboard the *open* price is the previous session's close, and the site's own
"% Change" column often reads 0. Market Miner always calculates `change = close − open` itself.

## API

All responses are JSON. Errors look like `{"error": "..."}`.

| Endpoint | Description |
|---|---|
| `GET /api/health` | Service status and the last scrape result |
| `GET /api/market/latest` | Latest snapshot: every quote, plus totals and advancers/decliners |
| `GET /api/market/movers?limit=5` | Top gainers, losers and most traded |
| `GET /api/securities` | All securities with first/last seen and data point counts |
| `GET /api/securities/{symbol}` | One security and its latest quote |
| `GET /api/securities/{symbol}/history?from=&to=&limit=` | Price history, oldest first. `from`/`to` are RFC 3339 timestamps |
| `GET /api/stream` | Server-Sent Events. Each `market` event carries the same JSON as `/api/market/latest` |

Example:

```sh
curl localhost:8080/api/market/movers?limit=3
```

## Configuration

Set in `.env` (see [`.env.example`](.env.example)) or the environment.

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | *(required)* | Postgres connection string. Docker Compose sets it for you |
| `BIND_ADDR` | `0.0.0.0:8080` | Address the server listens on |
| `SCRAPE_INTERVAL_SECS` | `300` | Seconds between scrapes (minimum 10) |
| `SCRAPER_ENABLED` | `true` | Set to `false` to serve the API without scraping |
| `MSE_URL` | `https://mse.co.mw/market/mainboard` | Page to scrape |
| `STATIC_DIR` | `web/dist` | Built dashboard to serve at `/` |
| `RUST_LOG` | `market_miner=info,tower_http=info` | Log filter |
| `APP_PORT`, `DB_PORT` | `8080`, `5433` | Host ports used by Docker Compose |

## Development

### Server (Rust)

With Rust installed:

```sh
docker compose up -d db                 # Postgres on localhost:5433
cp .env.example .env
cargo run
cargo test                              # needs the database for the DB and API tests
```

Without Rust installed, run the same checks CI runs inside a container:

```sh
docker compose --profile dev run --rm test   # fmt, clippy and tests
```

### Dashboard

```sh
cd web
npm install
npm run dev        # http://localhost:5173, proxies /api to localhost:8080
npm test
```

### Tests

- **Parser**: runs against [`tests/fixtures/mainboard.html`](tests/fixtures/mainboard.html), a trimmed copy of the
  real page. If MSE changes its layout, update the fixture and the parser together.
- **Database and API**: use `#[sqlx::test]`, which creates a throwaway database for each test from `DATABASE_URL`.
- **Dashboard**: Vitest and Testing Library.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
