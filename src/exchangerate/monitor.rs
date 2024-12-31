use serde::Deserialize;
use reqwest::Error;

pub struct ExchangeRateMonitor {
    previous_rate: Option<f64>,
}


impl ExchangeRateMonitor {
    pub fn new() -> Self {
        Self { previous_rate: None }
    }

    pub fn should_notify(&self, current_rate: f64, thresh: f64) -> bool {
        current_rate >= thresh
    }

    pub fn update_rate(&mut self, rate: f64) {
        self.previous_rate = Some(rate);
    }
}

#[derive(Debug, Deserialize)]
struct ExchangeRateResponse {
    conversion_rates: std::collections::HashMap<String, f64>,
    time_last_update_utc: String,
}

pub async fn fetch_exchange_rate(api_url: &str) -> Result<(f64, String), Error> {
    println!("request {api_url}");
    let response = reqwest::get(api_url).await?;

    if !response.status().is_success()  {
        eprintln!("Failed to fetch exchange rates: {}", response.status());
    }

    let exchange_data: ExchangeRateResponse = response.json().await?;
    println!("request done {}", exchange_data.time_last_update_utc.to_string());

    let sek_rate: f64 = exchange_data.conversion_rates.get("SEK").map(|v| v.to_owned()).unwrap();
    let date = exchange_data.time_last_update_utc;
    Ok((sek_rate, date))
}

