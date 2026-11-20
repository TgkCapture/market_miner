# Contributing

Thanks for helping out. Bug reports, fixes and ideas are all welcome.

## Getting set up

Follow **Development** in the [README](README.md). The quickest route is Docker only:

```sh
docker compose up -d --build                  # run the app
docker compose --profile dev run --rm test    # run the Rust checks
cd web && npm install && npm test             # run the dashboard tests
```

## Before opening a pull request

- `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.
- In `web/`: `npm run typecheck`, `npm test` and `npm run build` pass.
- New behaviour has a test. Parser changes should be tested against a fixture in `tests/fixtures/`.
- Database changes go in a **new** file in `migrations/`. Never edit a migration that has already been released.

CI runs all of the above on every pull request.

## Scraping politely

The scraper reads a public page on a small exchange's website. Keep the default interval (5 minutes) or longer,
and don't add features that send many requests in a short time.

## Reporting a bug

Please include what you expected, what happened, and the relevant lines from `docker compose logs app`.
If the scraper broke because the MSE page changed, attaching the page's HTML helps a lot.
