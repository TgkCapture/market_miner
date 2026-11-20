//! Turns the MSE mainboard HTML into a [`MarketSnapshot`]. Pure functions, no I/O.

use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use scraper::{ElementRef, Html, Selector};

use crate::market::{MarketSnapshot, MarketStatus, Quote};

/// Malawi is UTC+2 all year (no daylight saving).
const MALAWI_OFFSET_SECS: i32 = 2 * 3600;

static TABLE: LazyLock<Selector> = LazyLock::new(|| sel("table.table-sm"));
static ROW: LazyLock<Selector> = LazyLock::new(|| sel("tbody tr"));
static CELL: LazyLock<Selector> = LazyLock::new(|| sel("td"));
static LINK: LazyLock<Selector> = LazyLock::new(|| sel("a"));
static FOOTER: LazyLock<Selector> = LazyLock::new(|| sel("tfoot"));

fn sel(css: &str) -> Selector {
    Selector::parse(css).expect("static selector is valid")
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("price table not found; the page layout may have changed")]
    TableNotFound,
    #[error("price table had no readable rows")]
    NoRows,
    #[error("could not find the \"as at\" date on the page")]
    NoTimestamp,
}

pub fn parse_mainboard(html: &str) -> Result<MarketSnapshot, ParseError> {
    let doc = Html::parse_document(html);
    let table = doc.select(&TABLE).next().ok_or(ParseError::TableNotFound)?;

    let mut quotes = Vec::new();
    for row in table.select(&ROW) {
        match parse_row(row) {
            Some(quote) => quotes.push(quote),
            None => tracing::warn!(row = %text_of(row), "skipping unreadable price row"),
        }
    }
    if quotes.is_empty() {
        return Err(ParseError::NoRows);
    }

    // Prefer the table footer ("Stats as at 20/11/2026 15:00pm"); fall back to the header
    // ("Updated on: 20/11/26 15:00pm").
    let footer = table
        .select(&FOOTER)
        .next()
        .map(text_of)
        .unwrap_or_default();
    let as_of = find_timestamp(&footer)
        .or_else(|| find_timestamp(&text_of(doc.root_element())))
        .ok_or(ParseError::NoTimestamp)?;

    Ok(MarketSnapshot {
        as_of,
        status: find_status(&text_of(doc.root_element())),
        quotes,
    })
}

fn parse_row(row: ElementRef) -> Option<Quote> {
    let cells: Vec<ElementRef> = row.select(&CELL).collect();
    if cells.len() < 6 {
        return None;
    }
    let link = cells[0].select(&LINK).next();
    let symbol = link.map(text_of).unwrap_or_else(|| text_of(cells[0]));
    if symbol.is_empty() {
        return None;
    }
    let isin = link
        .and_then(|a| a.value().attr("href"))
        .and_then(|href| href.trim_end_matches('/').rsplit('/').next())
        .filter(|code| !code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric()))
        .map(str::to_string);

    let num = |i: usize| parse_number(&text_of(cells[i]));
    // cells[3] is the site's "% Change", which is unreliable; Quote::new calculates it.
    Some(Quote::new(symbol, isin, num(1)?, num(2)?, num(4)?, num(5)?))
}

fn text_of(el: ElementRef) -> String {
    el.text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parses numbers like `2,022,471,000.00`. Returns `None` for anything else.
pub fn parse_number(text: &str) -> Option<f64> {
    let cleaned: String = text.trim().chars().filter(|c| *c != ',').collect();
    cleaned.parse::<f64>().ok().filter(|n| n.is_finite())
}

/// Finds the first `DD/MM/YY[YY] HH:MM[am|pm]` in `text` and returns it in UTC.
///
/// The site writes 24-hour times with a stray suffix (`15:00pm`), so the suffix only
/// matters for hours below 13.
pub fn find_timestamp(text: &str) -> Option<DateTime<Utc>> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    tokens.windows(2).find_map(|w| {
        let date = parse_date(w[0])?;
        let time = parse_time(w[1])?;
        let local = FixedOffset::east_opt(MALAWI_OFFSET_SECS)?
            .from_local_datetime(&NaiveDateTime::new(date, time))
            .single()?;
        Some(local.with_timezone(&Utc))
    })
}

fn parse_date(token: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = token.split('/').collect();
    let [d, m, y] = parts.as_slice() else {
        return None;
    };
    let mut year: i32 = y.parse().ok()?;
    if y.len() == 2 {
        year += 2000;
    }
    NaiveDate::from_ymd_opt(year, m.parse().ok()?, d.parse().ok()?)
}

fn parse_time(token: &str) -> Option<NaiveTime> {
    let lower = token.to_ascii_lowercase();
    let (clock, suffix) = match lower.strip_suffix("pm") {
        Some(c) => (c, Some("pm")),
        None => match lower.strip_suffix("am") {
            Some(c) => (c, Some("am")),
            None => (lower.as_str(), None),
        },
    };
    let (h, m) = clock.split_once(':')?;
    let mut hour: u32 = h.parse().ok()?;
    let minute: u32 = m.parse().ok()?;
    match (suffix, hour) {
        (Some("pm"), 1..=11) => hour += 12,
        (Some("am"), 12) => hour = 0,
        _ => {}
    }
    NaiveTime::from_hms_opt(hour, minute, 0)
}

fn find_status(text: &str) -> MarketStatus {
    let Some((_, rest)) = text.split_once("Market Status:") else {
        return MarketStatus::Unknown;
    };
    match rest
        .split_whitespace()
        .next()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("open") => MarketStatus::Open,
        Some("closed") => MarketStatus::Closed,
        _ => MarketStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const FIXTURE: &str = include_str!("../../tests/fixtures/mainboard.html");

    fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    #[test]
    fn parses_real_page() {
        let snap = parse_mainboard(FIXTURE).unwrap();
        assert_eq!(snap.quotes.len(), 17);
        assert_eq!(snap.status, MarketStatus::Closed);
        // 15:00 in Blantyre is 13:00 UTC.
        assert_eq!(snap.as_of, utc(2026, 11, 20, 13, 0));

        let chl = snap.quotes.iter().find(|q| q.symbol == "CHL").unwrap();
        assert_eq!(chl.isin.as_deref(), Some("MWCHL0010175"));
        assert_eq!((chl.open, chl.close), (193.04, 194.91));
        assert_eq!(chl.change, 1.87);
        assert_eq!((chl.volume, chl.turnover), (15_076.0, 2_938_484.0));

        let fdhb = snap.quotes.iter().find(|q| q.symbol == "FDHB").unwrap();
        assert_eq!(fdhb.turnover, 2_022_471_000.0);
        assert_eq!(fdhb.change, -0.78);

        let omu = snap.quotes.iter().find(|q| q.symbol == "OMU").unwrap();
        assert_eq!(omu.isin.as_deref(), Some("ZAE000255360"));
    }

    #[test]
    fn missing_table_is_an_error() {
        assert_eq!(
            parse_mainboard("<html><body>maintenance</body></html>"),
            Err(ParseError::TableNotFound)
        );
    }

    #[test]
    fn empty_table_is_an_error() {
        let html = r#"<table class="table table-sm"><tbody></tbody></table>"#;
        assert_eq!(parse_mainboard(html), Err(ParseError::NoRows));
    }

    #[test]
    fn bad_rows_are_skipped() {
        let html = r##"
            <table class="table table-sm"><tbody>
              <tr><td><a href="https://mse.co.mw/company/MWX1">X</a></td><td>1</td><td>2</td><td>0</td><td>3</td><td>4</td></tr>
              <tr><td><a href="#">BAD</a></td><td>n/a</td><td>2</td><td>0</td><td>3</td><td>4</td></tr>
              <tr><td colspan="6">short row</td></tr>
            </tbody><tfoot><tr><td>Stats as at 01/02/2026 09:30am</td></tr></tfoot></table>"##;
        let snap = parse_mainboard(html).unwrap();
        assert_eq!(snap.quotes.len(), 1);
        assert_eq!(snap.quotes[0].isin.as_deref(), Some("MWX1"));
        assert_eq!(snap.status, MarketStatus::Unknown);
        assert_eq!(snap.as_of, utc(2026, 2, 1, 7, 30));
    }

    #[test]
    fn missing_timestamp_is_an_error() {
        let html = r#"<table class="table-sm"><tbody>
            <tr><td>X</td><td>1</td><td>2</td><td>0</td><td>3</td><td>4</td></tr></tbody></table>"#;
        assert_eq!(parse_mainboard(html), Err(ParseError::NoTimestamp));
    }

    #[test]
    fn parses_numbers() {
        assert_eq!(parse_number("2,022,471,000.00"), Some(2_022_471_000.0));
        assert_eq!(parse_number(" 17.40 "), Some(17.4));
        assert_eq!(parse_number("-"), None);
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("NaN"), None);
    }

    #[test]
    fn parses_timestamps() {
        assert_eq!(
            find_timestamp("Updated on: 20/11/26 15:00pm"),
            Some(utc(2026, 11, 20, 13, 0))
        );
        assert_eq!(
            find_timestamp("as at 05/01/2026 1:15pm"),
            Some(utc(2026, 1, 5, 11, 15))
        );
        assert_eq!(
            find_timestamp("as at 05/01/2026 12:05am"),
            Some(utc(2026, 1, 4, 22, 5))
        );
        assert_eq!(
            find_timestamp("as at 05/01/2026 12:05pm"),
            Some(utc(2026, 1, 5, 10, 5))
        );
        assert_eq!(find_timestamp("31/02/2026 10:00"), None);
        assert_eq!(find_timestamp("no date here"), None);
    }

    #[test]
    fn reads_market_status() {
        assert_eq!(find_status("Market Status: Open"), MarketStatus::Open);
        assert_eq!(find_status("Market Status: Closed"), MarketStatus::Closed);
        assert_eq!(find_status("nothing"), MarketStatus::Unknown);
    }
}
