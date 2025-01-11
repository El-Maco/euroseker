use std::fs;

use chrono::{DateTime, Local, Utc};
use reqwest::Error;
use serde::{Deserialize, Serialize};

use crate::utils::FileStorage;

use super::{love_note::LoveNote, plotter};

#[derive(Deserialize, Debug)]
pub struct ExchangeRateConfig {
    pub threshold: f64,
    pub debug: bool,
}

static CONFIG_FILE: &str = "config.json";
impl ExchangeRateConfig {
    pub fn new() -> Self {
        let config_data =
            fs::read_to_string(CONFIG_FILE).expect("Failed to read config: {CONFIG_FILE}");
        let config: ExchangeRateConfig =
            serde_json::from_str(&config_data).expect("Failed to parse ExchangeRateConfig");
        config
    }
}

pub struct ExchangeRateMonitor {
    storage: FileStorage,
}

impl ExchangeRateMonitor {
    pub fn new() -> Self {
        Self {
            storage: FileStorage::new("data.json"),
        }
    }

    pub fn should_notify(&self, current_rate: f64, thresh: f64) -> Option<String> {
        let love_note = LoveNote::new();
        println!("Love message: {}", love_note.message);
        if self.storage.history.len() >= 3 {
            let last_three = &self.storage.history[self.storage.history.len() - 3..];
            println!("last_three {:?}", last_three);
            if last_three[0].rate < last_three[1].rate && last_three[1].rate < last_three[2].rate {
                return Some(format!("Better EURO to SEK rate now than the previous 2 days. Gone from {:.4} -> {:.4} -> {:.4}\n\n{} ❤️\n\nLove, Maco 🥰", last_three[0].rate, last_three[1].rate, last_three[2].rate, love_note.message));
            }
        }
        if current_rate >= thresh {
            return Some(format!("The exchange rate has now exceeded the limit of {thresh:.2} SEK. The rate is now 1 EUR = {:.2} SEK\n\n{} ❤️\n\nLove, Maco 🥰", current_rate, love_note.message));
        }
        None
    }

    pub async fn fetch_exchange_rate(&mut self, api_url: &str) -> Result<ExchangeRate, Error> {
        println!("request {api_url}");
        let response = reqwest::get(api_url).await?;

        if !response.status().is_success() {
            eprintln!("Failed to fetch exchange rates: {}", response.status());
        }

        let exchange_data: ExchangeRateResponse = response.json().await?;
        println!(
            "request done {}",
            exchange_data.time_last_update_utc.to_string()
        );

        let sek_rate: f64 = exchange_data
            .conversion_rates
            .get("SEK")
            .map(|v| v.to_owned())
            .unwrap();
        let exchange_rate = ExchangeRate::new(sek_rate, &exchange_data.time_last_update_utc);
        self.storage.add(exchange_rate.clone());
        Ok(exchange_rate)
    }

    pub fn plot_rates(&self) -> Result<String, Box<dyn std::error::Error>> {
        let plot_path = format!("plots/exchangerate-{}.png", Local::now().format("%d%m%Y"));
        plotter::generate_plot(&self.storage.history, &plot_path)?;
        Ok(plot_path)
    }
}

#[derive(Debug, Deserialize)]
struct ExchangeRateResponse {
    conversion_rates: std::collections::HashMap<String, f64>,
    time_last_update_utc: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExchangeRate {
    pub rate: f64,
    pub date: DateTime<Utc>,
}

impl ExchangeRate {
    pub fn new(rate: f64, date: &str) -> Self {
        ExchangeRate {
            rate,
            date: DateTime::parse_from_rfc2822(date)
                .expect("Failed to parse datetime")
                .into(),
        }
    }
}
