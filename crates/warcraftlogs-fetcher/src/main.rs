use anyhow::{Context, Result};
use chrono::NaiveDate;
use headless_chrome::{Browser, FetcherOptions, LaunchOptions};
use scraper::{Html, Selector};
use std::time::Duration;

/// TBC Pre-patch period: May 18, 2021 - June 1, 2021
const TBC_PREPATCH_START: &str = "2021-05-18";
const TBC_PREPATCH_END: &str = "2021-06-01";

const START_URL: &str = "https://classic.warcraftlogs.com/zone/reports?zone=1006";

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

    let tab = browser.new_tab().context("Failed to create tab")?;

    println!("Navigating to {}...", START_URL);
    tab.navigate_to(START_URL)?;
    tab.wait_until_navigated()?;
    std::thread::sleep(Duration::from_secs(2));

    let mut page_num = 1;
    loop {
        println!("Checking page {}...", page_num);

        let html = tab.get_content().context("Failed to get page content")?;
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
            println!("\nCurrent URL: {}", tab.get_url());
            return Ok(());
        }

        if let Some(oldest) = oldest_date {
            println!("  Oldest date on page: {}", oldest);
            if oldest < prepatch_start {
                println!("\nReached logs older than pre-patch period. Stopping.");
                return Ok(());
            }
        } else {
            println!("  No dates found on this page");
        }

        // Try to click "Next" button
        match click_next(&tab) {
            Ok(()) => {
                std::thread::sleep(Duration::from_secs(2));
                page_num += 1;
            }
            Err(_) => {
                println!("\nNo more pages (couldn't find Next button).");
                return Ok(());
            }
        }
    }
}

fn click_next(tab: &headless_chrome::Tab) -> Result<()> {
    // Try various selectors for the "Next" pagination link
    let selectors = [
        "a.pagination-next",
        ".pagination a:contains('Next')",
        "a[rel='next']",
        ".paging a:last-child",
        "a.next",
    ];

    for selector in &selectors {
        if let Ok(elem) = tab.find_element(selector) {
            elem.click()?;
            tab.wait_until_navigated()?;
            return Ok(());
        }
    }

    // Fallback: find any link containing "Next" text
    let html = tab.get_content()?;
    let document = Html::parse_document(&html);
    let link_selector = Selector::parse("a").unwrap();

    for link in document.select(&link_selector) {
        let text = link.text().collect::<String>();
        if text.to_lowercase().contains("next") {
            // Get href and navigate
            if let Some(href) = link.value().attr("href") {
                let url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("https://classic.warcraftlogs.com{}", href)
                };
                tab.navigate_to(&url)?;
                tab.wait_until_navigated()?;
                return Ok(());
            }
        }
    }

    anyhow::bail!("Could not find Next button")
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

    let row_selector =
        Selector::parse("table tbody tr, .report-overview-table tr").expect("valid selector");

    for row in document.select(&row_selector) {
        let text = row.text().collect::<String>();

        if let Some(date) = try_parse_date_from_text(&text) {
            oldest_date = Some(oldest_date.map_or(date, |d: NaiveDate| d.min(date)));

            if date >= start && date <= end {
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
