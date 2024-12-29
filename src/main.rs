use serde::Deserialize;
use reqwest::Error;
use tokio;
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Deserialize)]
struct ExchangeRateResponse {
    conversion_rates: std::collections::HashMap<String, f64>,
    time_last_update_utc: String,
}

async fn get_eur_to_sek(api_url: &str) -> Result<(f64, String), Error> {
    println!("request {api_url}");
    let response = reqwest::get(api_url).await?;

    if !response.status().is_success()  {
        eprintln!("Failed to fetch exchange rates: {}", response.status());
    }

    println!("Before");
    let exchange_data: ExchangeRateResponse = response.json().await?;
    println!("request done {}", exchange_data.time_last_update_utc.to_string());
    println!("after");

    let sek_rate: f64 = exchange_data.conversion_rates.get("SEK").map(|v| v.to_owned()).unwrap();
    let date = exchange_data.time_last_update_utc;
    Ok((sek_rate, date))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let api_key = env::var("EXCHANGE_API_TOKEN").expect("EXCHANGE_API_TOKEN not found");
    let api_url = format!("https://v6.exchangerate-api.com/v6/{}/latest/EUR", api_key);

    let (sek_rate, date) = get_eur_to_sek(&api_url).await?;

    println!("{:?}: 1 EUR = {} SEK", date, sek_rate);

    Ok(())
}
