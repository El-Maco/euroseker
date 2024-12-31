use serde::Deserialize;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use std::env;

#[derive(Debug, Deserialize)]
pub struct EmailMessage {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub fn send_email(message: EmailMessage) {
    let email = Message::builder()
        .from(message.from.parse().unwrap())
        .to(message.to.parse().unwrap())
        .subject(message.subject)
        .body(message.body)
        .unwrap();

    let password = env::var("EMAIL_PASS").expect("EMAIL_PASS not found");
    let creds = Credentials::new(message.from, password);

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email send successfully."),
        Err(e) => eprintln!("Failed to send email: {}", e)
    }

}

