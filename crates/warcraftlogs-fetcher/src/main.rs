use anyhow::{Context, Result};
use chrono::NaiveDate;
use headless_chrome::{Browser, FetcherOptions, LaunchOptions};
use scraper::{Html, Selector};
use std::time::Duration;

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

    println!("Launching browser (will download Chromium if needed)...");

    let browser = Browser::new(LaunchOptions {
        headless: true,
        fetcher_options: FetcherOptions::default().with_allow_download(true),
        ..Default::default()
    })
    .context("Failed to launch browser")?;

    let mut page_num = 1;
    loop {
        let url = format!("{}?zone={}&page={}", BASE_URL, ZONE_ID, page_num);
        println!("Fetching page {}...", page_num);

        let tab = browser.new_tab().context("Failed to create new tab")?;
        tab.navigate_to(&url)
            .context(format!("Failed to navigate to {}", url))?;

        // Wait for page to load and JS to execute
        std::thread::sleep(Duration::from_secs(3));

        // Wait for the table to appear
        let _ = tab.wait_for_element("table");

        let html = tab
            .get_content()
            .context("Failed to get page content")?;

        let document = Html::parse_document(&html);
        let (logs, oldest_date) = parse_logs(&document, prepatch_start, prepatch_end)?;

        if !logs.is_empty() {
            println!(
                "\nFound {} logs from TBC pre-patch period on page {}:",
                logs.len(),
                page_num
            );
            for log in &logs {
                println!("  - {} | {}", log.date, log.title);
            }
            println!("\nFirst matching page: {}", page_num);
            println!("URL: {}", url);
            return Ok(());
        }

        // Check if we've gone past the prepatch period (logs are in reverse chronological order)
        if let Some(oldest) = oldest_date {
            println!("  Oldest date on page: {}", oldest);
            if oldest < prepatch_start {
                println!("\nReached logs older than pre-patch period. No logs found.");
                return Ok(());
            }
        } else {
            println!("  No dates found on this page");
        }

        // Check if there are more pages
        if !has_next_page(&document) {
            println!("\nNo more pages. No logs found from pre-patch period.");
            return Ok(());
        }

        page_num += 1;

        // Rate limiting
        std::thread::sleep(Duration::from_secs(1));
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

    // WarcraftLogs uses table rows for reports
    // Try multiple selectors to find the right structure
    let row_selector =
        Selector::parse("table tbody tr, .report-overview-table tr").expect("valid selector");

    for row in document.select(&row_selector) {
        let text = row.text().collect::<String>();

        if let Some(date) = try_parse_date_from_text(&text) {
            oldest_date = Some(oldest_date.map_or(date, |d: NaiveDate| d.min(date)));

            if date >= start && date <= end {
                // Extract title from link
                let title = row
                    .select(&Selector::parse("a").unwrap())
                    .next()
                    .map(|a| a.text().collect::<String>())
                    .unwrap_or_else(|| "Unknown".to_string());

                matching_logs.push(LogEntry {
                    title: title.trim().to_string(),
                    date,
                });
            }
        }
    }

    Ok((matching_logs, oldest_date))
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

    // Try to extract date pattern from text using regex
    let date_patterns = [
        (r"(\d{4}-\d{2}-\d{2})", "%Y-%m-%d"),
        (r"(\d{1,2}/\d{1,2}/\d{4})", "%m/%d/%Y"),
        (r"(\d{1,2}/\d{1,2}/\d{2})", "%m/%d/%y"),
    ];

    for (pattern, fmt) in &date_patterns {
        if let Ok(re) = regex_lite::Regex::new(pattern) {
            if let Some(cap) = re.captures(text) {
                if let Some(m) = cap.get(1) {
                    if let Ok(date) = NaiveDate::parse_from_str(m.as_str(), fmt) {
                        return Some(date);
                    }
                }
            }
        }
    }

    None
}

fn has_next_page(document: &Html) -> bool {
    // Check for pagination - look for "Next" link or page numbers
    if let Ok(selector) = Selector::parse("a") {
        for a in document.select(&selector) {
            let text = a.text().collect::<String>().to_lowercase();
            if text.contains("next") || text.contains("›") || text.contains("»") {
                return true;
            }
        }
    }
    false
}
