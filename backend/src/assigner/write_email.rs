use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

pub async fn send_email(
    body: String,
    recipients: &[String],
) -> Result<(), lettre::error::Error> {
    // TODO: Either remove .expects here, or have this run on server startup
    let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST not set");
    let smtp_port: u16 = std::env::var("SMTP_PORT")
        .expect("SMTP_PORT not set")
        .parse()
        .expect("SMTP_PORT must be a number");
    let smtp_user = std::env::var("SMTP_USER").expect("SMTP_USER not set");
    let smtp_password = std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not set");
    let from_address = std::env::var("SMTP_FROM").expect("SMTP_FROM not set");

    let creds = Credentials::new(smtp_user, smtp_password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
        .unwrap() // TODO: Remove naked unwraps
        .port(smtp_port) // TODO Lettere says you don't need to use this... Find out if it is necessary to use
        .credentials(creds)
        .build();

    for recipient in recipients {
        let email = Message::builder()
            .from(from_address.parse().unwrap())
            .to(recipient.parse().unwrap())
            .subject("Challenge notification")
            .header(ContentType::TEXT_PLAIN)
            .body(body.clone())
            .unwrap();

        mailer.send(email).await.unwrap();
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let body = "Hello, this is a test email.".to_string();
    let recipients = vec![
        "karlmeisner99@gmail.com".to_string(),
        "kam@unf.com".to_string(),
    ];

    send_email(body, &recipients).await.unwrap();
    println!("Emails sent successfully.");
}
