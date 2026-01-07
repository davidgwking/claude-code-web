use anyhow::{Context, Result};
use chrono::NaiveDate;
use scraper::{Html, Selector};
use std::time::Duration;

/// TBC Pre-patch period: May 18, 2021 - June 1, 2021
const TBC_PREPATCH_START: &str = "2021-05-18";
const TBC_PREPATCH_END: &str = "2021-06-01";

const START_URL: &str = "https://classic.warcraftlogs.com/zone/reports?zone=1006";
const BASE_URL: &str = "https://classic.warcraftlogs.com";

fn main() -> Result<()> {
    let prepatch_start = NaiveDate::parse_from_str(TBC_PREPATCH_START, "%Y-%m-%d")?;
    let prepatch_end = NaiveDate::parse_from_str(TBC_PREPATCH_END, "%Y-%m-%d")?;

    println!("Searching for WoW Classic logs from TBC pre-patch period");
    println!("Period: {} to {}", TBC_PREPATCH_START, TBC_PREPATCH_END);
    println!();

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .cookie_store(true)
        .gzip(true)
        .build()?;

    let mut current_url = START_URL.to_string();
    let mut page_num = 1;

    loop {
        println!("Fetching page {}...", page_num);
        println!("  URL: {}", current_url);

        let response = client
            .get(&current_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Cache-Control", "max-age=0")
            .send()
            .context(format!("Failed to fetch {}", current_url))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP error: {}", status);
        }

        let html = response.text()?;
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
            println!("\nURL: {}", current_url);
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

        // Find "Next" link
        match find_next_url(&document) {
            Some(next_url) => {
                current_url = next_url;
                page_num += 1;
                std::thread::sleep(Duration::from_millis(500));
            }
            None => {
                println!("\nNo more pages (couldn't find Next link).");
                return Ok(());
            }
        }
    }
}

fn find_next_url(document: &Html) -> Option<String> {
    let link_selector = Selector::parse("a").ok()?;

    for link in document.select(&link_selector) {
        let text = link.text().collect::<String>().to_lowercase();
        if text.contains("next") || text.trim() == "›" || text.trim() == "»" {
            if let Some(href) = link.value().attr("href") {
                return Some(if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", BASE_URL, href)
                });
            }
        }
    }
    None
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
        Selector::parse("table tbody tr").expect("valid selector");

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
