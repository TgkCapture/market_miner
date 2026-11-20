-- Listed securities. One row per ticker symbol.
CREATE TABLE securities (
    id          BIGSERIAL PRIMARY KEY,
    symbol      TEXT        NOT NULL UNIQUE,
    isin        TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Price snapshots. One row per security per "stats as at" time published by the exchange,
-- so re-scraping an unchanged page never creates duplicates.
CREATE TABLE quotes (
    id           BIGSERIAL PRIMARY KEY,
    security_id  BIGINT           NOT NULL REFERENCES securities (id) ON DELETE CASCADE,
    as_of        TIMESTAMPTZ      NOT NULL,
    open_price   DOUBLE PRECISION NOT NULL,
    close_price  DOUBLE PRECISION NOT NULL,
    volume       DOUBLE PRECISION NOT NULL,
    turnover     DOUBLE PRECISION NOT NULL,
    scraped_at   TIMESTAMPTZ      NOT NULL DEFAULT now(),
    UNIQUE (security_id, as_of)
);

CREATE INDEX quotes_security_as_of_idx ON quotes (security_id, as_of DESC);
CREATE INDEX quotes_as_of_idx ON quotes (as_of DESC);
