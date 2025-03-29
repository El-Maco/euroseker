mod exchangerate;
mod utils;

use dotenvy::dotenv;
use exchangerate::email::{send_email, EmailMessage};
use exchangerate::monitor::{ExchangeRateConfig, ExchangeRateMonitor};
use std::{env, time::Duration};
use tokio::{self, time};


#[tokio::main]
async fn main() {
    dotenv().ok();

    let api_key = env::var("EXCHANGE_API_TOKEN").expect("EXCHANGE_API_TOKEN not found");
    let api_url = format!("https://v6.exchangerate-api.com/v6/{}/latest/EUR", api_key);

    let mut monitor = ExchangeRateMonitor::new();

    let config = ExchangeRateConfig::new();

    loop {
        match monitor.fetch_exchange_rate(&api_url).await {
            Ok(exchange_rate) => {
                        let from = env::var("FROM_EMAIL").expect("FROM_EMAIL not found");
                        let to = env::var("TO_EMAILS").ok();
                        let cc = env::var("CC_EMAILS").ok();
                        let bcc = env::var("BCC_EMAILS").unwrap_or(String::new()).split(";").map(|s| s.to_string()).collect();
                if let Some(body) = monitor.should_notify(exchange_rate.rate, config.threshold) {
                    let email_message: EmailMessage = EmailMessage {
                        from, to, cc, bcc,
                        subject: "[Aγάπη σου ❤️] Exchange Rate Alert".to_string(),
                        body,
                        attachment: monitor.plot_rates().ok(),
                    };
                    println!("Sending email {:?}", email_message);
                    send_email(email_message, config.debug);
                }

                println!(
                    "{:?}: 1 EUR = {} SEK",
                    exchange_rate.date, exchange_rate.rate
                );

                time::sleep(Duration::from_secs(24 * 60 * 60)).await;
            }
            Err(e) => eprintln!("Failed to fetch exchange rate: {}", e),
        }
    }
}
