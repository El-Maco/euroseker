use serde::{Deserialize, Serialize};
use reqwest::Error;
use chrono::{DateTime, Utc};

use crate::utils::FileStorage;

pub struct ExchangeRateMonitor {
    previous_rate: Option<f64>,
    storage: FileStorage,
}


impl ExchangeRateMonitor {
    pub fn new() -> Self {
        Self { previous_rate: None, storage: FileStorage::new("data.json") }
    }

    pub fn should_notify(&self, current_rate: f64, thresh: f64) -> Option<String> {
        if self.storage.history.len() >= 3 {
            let last_three = &self.storage.history[self.storage.history.len() - 3..];
            println!("last_three {:?}", last_three);
            if last_three[0].rate < last_three[1].rate && last_three[1].rate < last_three[2].rate {
                return Some(format!("Better EURO to SEK rate now than the previous 2 days. Gone from {:.4} -> {:.4} -> {:.4}", last_three[0].rate, last_three[1].rate, last_three[2].rate))
            }
        }
        if current_rate >= thresh {
            return Some(format!("The exchange rate has now exceeded the limit of {thresh} SEK. The rate is now 1 EUR = {:.2} SEK", current_rate));
        }
        None
    }

    pub fn update_rate(&mut self, rate: f64) {
        self.previous_rate = Some(rate);
    }

    pub async fn fetch_exchange_rate(&mut self, api_url: &str) -> Result<ExchangeRate, Error> {
        println!("request {api_url}");
        let response = reqwest::get(api_url).await?;

        if !response.status().is_success()  {
            eprintln!("Failed to fetch exchange rates: {}", response.status());
        }

        let exchange_data: ExchangeRateResponse = response.json().await?;
        println!("request done {}", exchange_data.time_last_update_utc.to_string());


        let sek_rate: f64 = exchange_data.conversion_rates.get("SEK").map(|v| v.to_owned()).unwrap();
        let date: DateTime<Utc> = DateTime::parse_from_rfc2822(&exchange_data.time_last_update_utc).expect("Invalid RFC2822 Date").into();
        let exchange_rate = ExchangeRate{ rate: sek_rate, date };
        self.storage.add(exchange_rate.clone());
        Ok(exchange_rate)
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

