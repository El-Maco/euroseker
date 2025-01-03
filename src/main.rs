use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use reqwest::Error;
use tokio::{self, time};
use dotenvy::dotenv;
use std::{env, time::Duration};

#[derive(Debug, Deserialize)]
struct ExchangeRateResponse {
    conversion_rates: std::collections::HashMap<String, f64>,
    time_last_update_utc: String,
}

struct ExchangeRateMonitor {
    previous_rate: Option<f64>,
}


impl ExchangeRateMonitor {
    fn new() -> Self {
        Self { previous_rate: None }
    }

    fn should_notify(&self, current_rate: f64, thresh: f64) -> bool {
        current_rate >= thresh
    }

    fn update_rate(&mut self, rate: f64) {
        self.previous_rate = Some(rate);
    }
}

#[derive(Debug, Deserialize)]
struct EmailMessage {
    from: String,
    to: String,
    subject: String,
    body: String,
}

fn send_email(message: EmailMessage) {
    let email = Message::builder()
        .from(message.from.parse().unwrap())
        .to(message.to.parse().unwrap())
        .subject(message.subject)
        .body(message.body)
        .unwrap();

    let password = env::var("EMAIL_PASS").expect("EMAIL_PASS not found");
    let creds = Credentials::new(from_email, password);

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email send successfully."),
        Err(e) => eprintln!("Failed to send email: {}", e)
    }

}

async fn fetch_exchange_rate(api_url: &str) -> Result<(f64, String), Error> {
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
async fn main() {
    dotenv().ok();

    let api_key = env::var("EXCHANGE_API_TOKEN").expect("EXCHANGE_API_TOKEN not found");
    let api_url = format!("https://v6.exchangerate-api.com/v6/{}/latest/EUR", api_key);

    let mut monitor = ExchangeRateMonitor::new();

    loop {

        match fetch_exchange_rate(&api_url).await {
            Ok((sek_rate, date)) =>  {
                // if monitor.should_notify(sek_rate, 11.55) {
                let email_message: EmailMessage = EmailMessage {
                    from: env::var("FROM_EMAIL").expect("FROM_EMAIL not found"),
                    to: env::var("TO_EMAILS").expect("TO_EMAILS not found"),
                    subject: "[Aγάπη σου ❤️] Exchange Rate Alert".to_string(),
                    body: format!("The exchange rate is now 1 EUR = {:.2} SEK", sek_rate),
                };
                if monitor.should_notify(sek_rate, 11.55) {
                    println!("Criteria met, sending notification");
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
