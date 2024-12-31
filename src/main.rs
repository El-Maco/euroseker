mod exchangerate;

use exchangerate::monitor::{ExchangeRateMonitor, fetch_exchange_rate};
use exchangerate::email::{EmailMessage, send_email};
use tokio::{self, time};
use dotenvy::dotenv;
use std::{env, time::Duration};


#[tokio::main]
async fn main() {
    dotenv().ok();

    let api_key = env::var("EXCHANGE_API_TOKEN").expect("EXCHANGE_API_TOKEN not found");
    let api_url = format!("https://v6.exchangerate-api.com/v6/{}/latest/EUR", api_key);

    let mut monitor = ExchangeRateMonitor::new();

    loop {

        match fetch_exchange_rate(&api_url).await {
            Ok((sek_rate, date)) =>  {
                let email_message: EmailMessage = EmailMessage {
                    from: env::var("FROM_EMAIL").expect("FROM_EMAIL not found"),
                    to: env::var("TO_EMAILS").expect("TO_EMAILS not found"),
                    subject: "[Aγάπη σου ❤️] Exchange Rate Alert".to_string(),
                    body: format!("The exchange rate is now 1 EUR = {:.2} SEK", sek_rate),
                };
                if monitor.should_notify(sek_rate, 0.0) {
                    println!("Criteria met, sending notification: {:?}", email_message);
                    send_email(email_message);
                }

                monitor.update_rate(sek_rate);

                println!("{:?}: 1 EUR = {} SEK", date, sek_rate);

                time::sleep(Duration::from_secs(24 * 60 * 60)).await;
            },
            Err(e) => eprintln!("Failed to fetch exchange rate: {}", e)
        }

    }
}
