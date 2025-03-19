// scraper/scraper.rs
use reqwest::Error;
use scraper::{Html, Selector};
use crate::models::stock::Stock;
use chrono::Utc;

/// Fetches stock data from the given URL and parses it into a vector of `Stock` structs.
pub async fn fetch_stock_data(url: &str) -> Result<Vec<Stock>, Error> {
    // Fetch the HTML content
    let response = reqwest::get(url).await?;
    let body = response.text().await?;

    let document = Html::parse_document(&body);

    // Define selectors for the table, rows, and columns
    let table_selector = Selector::parse("table.table-sm").unwrap();
    let row_selector = Selector::parse("tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut stocks = Vec::new();

    // Iterate over each table
    for table in document.select(&table_selector) {
        for row in table.select(&row_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();

            if cells.len() >= 6 {
                let symbol = cells[0]
                    .select(&Selector::parse("a").unwrap())
                    .next()
                    .and_then(|e| e.text().next())
                    .unwrap_or("N/A")
                    .trim()
                    .to_string();

                let open_price = parse_number(cells[1].text().collect::<String>());
                let close_price = parse_number(cells[2].text().collect::<String>());
                let percent_change = parse_number(cells[3].text().collect::<String>());
                let volume = parse_number(cells[4].text().collect::<String>());
                let turnover = parse_number(cells[5].text().collect::<String>());

                let timestamp = Utc::now();

                // Create a new `Stock` 
                stocks.push(Stock {
                    id: 0, 
                    symbol,
                    open_price,
                    close_price,
                    percent_change,
                    volume,
                    turnover,
                    timestamp,
                });
            }
        }
    }

    Ok(stocks)
}

/// Helper function to parse numbers with commas
fn parse_number(text: String) -> f64 {
    text.replace(",", "").parse::<f64>().unwrap_or(0.0)
}