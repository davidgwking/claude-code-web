use anyhow::{Context, Result};
use chrono::NaiveDate;
use scraper::{Html, Selector};

/// TBC Pre-patch period: May 18, 2021 - June 1, 2021
const TBC_PREPATCH_START: &str = "2021-05-18";
const TBC_PREPATCH_END: &str = "2021-06-01";

const BASE_URL: &str = "https://classic.warcraftlogs.com/zone/reports";
const ZONE_ID: u32 = 1006; // Naxxramas

fn main() -> Result<()> {
    let prepatch_start = NaiveDate::parse_from_str(TBC_PREPATCH_START, "%Y-%m-%d")?;
    let prepatch_end = NaiveDate::parse_from_str(TBC_PREPATCH_END, "%Y-%m-%d")?;

    println!("Searching for WoW Classic logs from TBC pre-patch period");
    println!("Period: {} to {}", TBC_PREPATCH_START, TBC_PREPATCH_END);
    println!();

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let mut page = 1;
    loop {
        let url = format!("{}?zone={}&page={}", BASE_URL, ZONE_ID, page);
        println!("Fetching page {}...", page);

        let response = client
            .get(&url)
            .send()
            .context(format!("Failed to fetch page {}", page))?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let html = response.text()?;
        let document = Html::parse_document(&html);

        let (logs, oldest_date) = parse_logs(&document, prepatch_start, prepatch_end)?;

        if !logs.is_empty() {
            println!("\nFound {} logs from TBC pre-patch period on page {}:", logs.len(), page);
            for log in &logs {
                println!("  - {} | {}", log.date, log.title);
            }
            println!("\nFirst matching page: {}", page);
            println!("URL: {}", url);
            return Ok(());
        }

        // Check if we've gone past the prepatch period (logs are in reverse chronological order)
        if let Some(oldest) = oldest_date {
            if oldest < prepatch_start {
                println!("\nReached logs older than pre-patch period. No logs found.");
                return Ok(());
            }
        }

        // Check if there are more pages
        if !has_next_page(&document) {
            println!("\nNo more pages. No logs found from pre-patch period.");
            return Ok(());
        }

        page += 1;

        // Rate limiting
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[derive(Debug)]
struct LogEntry {
    title: String,
    date: NaiveDate,
}

fn parse_logs(
    document: &Html,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<(Vec<LogEntry>, Option<NaiveDate>)> {
    let mut matching_logs = Vec::new();
    let mut oldest_date: Option<NaiveDate> = None;

    // WarcraftLogs uses a table with class "report-overview-table" or similar
    // Each row contains date and report info
    let row_selector = Selector::parse("tr.report-overview-row, table.report-overview-table tr")
        .expect("valid selector");
    let date_selector = Selector::parse("td.report-overview-date, .report-date, td:first-child")
        .expect("valid selector");
    let title_selector = Selector::parse("td.report-overview-title a, .report-title a, td a")
        .expect("valid selector");

    // Alternative: try parsing from the description list format
    let alt_selector = Selector::parse(".zone-report-row, .report-row").expect("valid selector");

    // First try table format
    for row in document.select(&row_selector) {
        if let Some((date, title)) = extract_log_info(&row, &date_selector, &title_selector) {
            oldest_date = Some(oldest_date.map_or(date, |d: NaiveDate| d.min(date)));

            if date >= start && date <= end {
                matching_logs.push(LogEntry { title, date });
            }
        }
    }

    // If no results, try alternative format
    if matching_logs.is_empty() && oldest_date.is_none() {
        for row in document.select(&alt_selector) {
            let text = row.text().collect::<String>();
            if let Some(date) = try_parse_date_from_text(&text) {
                oldest_date = Some(oldest_date.map_or(date, |d: NaiveDate| d.min(date)));

                if date >= start && date <= end {
                    let title = row
                        .select(&Selector::parse("a").unwrap())
                        .next()
                        .map(|a| a.text().collect::<String>())
                        .unwrap_or_else(|| text.clone());
                    matching_logs.push(LogEntry { title, date });
                }
            }
        }
    }

    Ok((matching_logs, oldest_date))
}

fn extract_log_info(
    row: &scraper::ElementRef,
    date_sel: &Selector,
    title_sel: &Selector,
) -> Option<(NaiveDate, String)> {
    let date_elem = row.select(date_sel).next()?;
    let date_text = date_elem.text().collect::<String>();
    let date = try_parse_date_from_text(&date_text)?;

    let title_elem = row.select(title_sel).next()?;
    let title = title_elem.text().collect::<String>();

    Some((date, title.trim().to_string()))
}

fn try_parse_date_from_text(text: &str) -> Option<NaiveDate> {
    // Try various date formats used by WarcraftLogs
    let formats = [
        "%Y-%m-%d",
        "%m/%d/%Y",
        "%m/%d/%y",
        "%B %d, %Y",
        "%b %d, %Y",
        "%d %B %Y",
        "%d %b %Y",
    ];

    let text = text.trim();

    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(text, fmt) {
            return Some(date);
        }
    }

    // Try to extract date pattern from text
    let date_patterns = [
        r"(\d{4}-\d{2}-\d{2})",
        r"(\d{1,2}/\d{1,2}/\d{4})",
        r"(\d{1,2}/\d{1,2}/\d{2})",
    ];

    for pattern in &date_patterns {
        if let Ok(re) = regex_lite::Regex::new(pattern) {
            if let Some(cap) = re.captures(text) {
                if let Some(m) = cap.get(1) {
                    for fmt in &formats {
                        if let Ok(date) = NaiveDate::parse_from_str(m.as_str(), fmt) {
                            return Some(date);
                        }
                    }
                }
            }
        }
    }

    None
}

fn has_next_page(document: &Html) -> bool {
    // Check for pagination links
    let next_selector =
        Selector::parse("a.pagination-next, .pagination a[rel='next'], a:contains('Next')")
            .unwrap_or_else(|_| Selector::parse("a").unwrap());

    document
        .select(&next_selector)
        .any(|a| a.text().collect::<String>().to_lowercase().contains("next"))
}
