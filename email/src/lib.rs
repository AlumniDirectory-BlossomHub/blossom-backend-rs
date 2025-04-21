use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::Tokio1Executor;
use std::ops::Deref;
use tera::Tera;

pub struct EmailBackend {
    pub transport: AsyncSmtpTransportTokio,
    pub host: String,
    pub from: Mailbox,
}

pub type AsyncSmtpTransportTokio = AsyncSmtpTransport<Tokio1Executor>;

impl EmailBackend {
    pub fn from_env() -> Self {
        let host = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let username = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
        let password = std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        let from = format!("Blossom Dev Test <{}>", &username);
        let transport = build_transport(host.as_str(), username, password);
        Self {
            transport,
            host,
            from: from.parse().unwrap(),
        }
    }
}

impl Deref for EmailBackend {
    type Target = AsyncSmtpTransportTokio;
    fn deref(&self) -> &Self::Target {
        &self.transport
    }
}

pub fn build_transport(host: &str, username: String, password: String) -> AsyncSmtpTransportTokio {
    let creds = Credentials::new(username, password);
    AsyncSmtpTransport::<Tokio1Executor>::relay(host)
        .unwrap()
        .credentials(creds)
        .build()
}

pub fn templates() -> tera::Tera {
    match Tera::new("templates/email/*") {
        Ok(t) => t,
        Err(e) => {
            panic!("Error parsing templates: {}", e);
        }
    }
}
