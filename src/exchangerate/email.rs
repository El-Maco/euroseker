use serde::Deserialize;
use lettre::{message::{header::ContentType, Attachment, MultiPart, SinglePart}, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use std::{env, fs};

#[derive(Debug, Deserialize)]
pub struct EmailMessage {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub attachment: Option<String>,
}

fn create_attachment(path: Option<String>) -> SinglePart {
    match path {
        Some(attachment_path) => {
            let filebody = fs::read(attachment_path).expect("Failed to read attachment {attachment_path}");
            let content_type = ContentType::parse("image/png").unwrap();
            let attachment = Attachment::new("ExchangeRate".to_string()).body(filebody, content_type);
            attachment
        },
        None => SinglePart::plain("No plot found...".to_string()),
    }
}

pub fn send_email(message: EmailMessage) {
    let email = Message::builder()
        .from(message.from.parse().unwrap())
        .to(message.to.parse().unwrap())
        .subject(message.subject)
        .multipart(
            MultiPart::mixed()
            .singlepart(SinglePart::plain(message.body))
            .singlepart(create_attachment(message.attachment))
        )
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

