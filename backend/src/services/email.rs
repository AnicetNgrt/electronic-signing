use anyhow::Result;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tracing::{error, info};

use crate::services::config::Config;

pub struct EmailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
    from_name: String,
    public_url: String,
}

impl EmailService {
    pub fn new(config: &Config) -> Result<Self> {
        let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

        let transport = if config.smtp_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        };

        Ok(Self {
            transport,
            from_email: config.smtp_from_email.clone(),
            from_name: config.smtp_from_name.clone(),
            public_url: config.public_url.clone(),
        })
    }

    pub async fn send_signing_request(
        &self,
        to_email: &str,
        to_name: &str,
        document_title: &str,
        sender_name: &str,
        access_token: &str,
    ) -> Result<()> {
        let signing_url = format!("{}/sign/{}", self.public_url, access_token);

        let subject = format!(
            "{} has requested your signature on \"{}\"",
            sender_name, document_title
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Signature Request</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background-color: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
        <h1 style="color: #2563eb; margin: 0 0 10px 0; font-size: 24px;">Signature Request</h1>
        <p style="margin: 0; color: #666;">You have received a document to sign</p>
    </div>

    <p>Hello {to_name},</p>

    <p><strong>{sender_name}</strong> has requested your electronic signature on the following document:</p>

    <div style="background-color: #e8f4fd; padding: 15px; border-radius: 8px; margin: 20px 0;">
        <p style="margin: 0; font-weight: bold; color: #1e40af;">{document_title}</p>
    </div>

    <p>Please click the button below to review and sign the document:</p>

    <div style="text-align: center; margin: 30px 0;">
        <a href="{signing_url}" style="background-color: #2563eb; color: white; padding: 14px 28px; text-decoration: none; border-radius: 6px; font-weight: bold; display: inline-block;">Review & Sign Document</a>
    </div>

    <p style="font-size: 14px; color: #666;">If the button doesn't work, copy and paste this link into your browser:</p>
    <p style="font-size: 12px; color: #888; word-break: break-all;">{signing_url}</p>

    <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">

    <p style="font-size: 12px; color: #888;">
        This is an automated message from {from_name}. Please do not reply to this email.
        <br><br>
        Your electronic signature is legally binding under the ESIGN Act (USA) and eIDAS Regulation (EU).
    </p>
</body>
</html>"#,
            to_name = to_name,
            sender_name = sender_name,
            document_title = document_title,
            signing_url = signing_url,
            from_name = self.from_name
        );

        let plain_body = format!(
            r#"Signature Request

Hello {to_name},

{sender_name} has requested your electronic signature on the following document:

{document_title}

Please visit the following link to review and sign the document:
{signing_url}

Your electronic signature is legally binding under the ESIGN Act (USA) and eIDAS Regulation (EU).

---
This is an automated message from {from_name}. Please do not reply to this email."#,
            to_name = to_name,
            sender_name = sender_name,
            document_title = document_title,
            signing_url = signing_url,
            from_name = self.from_name
        );

        self.send_email(to_email, to_name, &subject, &html_body, &plain_body)
            .await
    }

    pub async fn send_completion_notification(
        &self,
        to_email: &str,
        to_name: &str,
        document_title: &str,
    ) -> Result<()> {
        let subject = format!("Document \"{}\" has been fully signed", document_title);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document Completed</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background-color: #d4edda; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
        <h1 style="color: #155724; margin: 0 0 10px 0; font-size: 24px;">Document Completed</h1>
        <p style="margin: 0; color: #155724;">All parties have signed the document</p>
    </div>

    <p>Hello {to_name},</p>

    <p>Great news! The following document has been signed by all parties:</p>

    <div style="background-color: #e8f4fd; padding: 15px; border-radius: 8px; margin: 20px 0;">
        <p style="margin: 0; font-weight: bold; color: #1e40af;">{document_title}</p>
    </div>

    <p>You can download the signed document and certificate of completion from your dashboard.</p>

    <div style="text-align: center; margin: 30px 0;">
        <a href="{dashboard_url}" style="background-color: #28a745; color: white; padding: 14px 28px; text-decoration: none; border-radius: 6px; font-weight: bold; display: inline-block;">View Dashboard</a>
    </div>

    <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">

    <p style="font-size: 12px; color: #888;">
        This is an automated message from {from_name}. Please do not reply to this email.
    </p>
</body>
</html>"#,
            to_name = to_name,
            document_title = document_title,
            dashboard_url = self.public_url,
            from_name = self.from_name
        );

        let plain_body = format!(
            r#"Document Completed

Hello {to_name},

Great news! The following document has been signed by all parties:

{document_title}

You can download the signed document and certificate of completion from your dashboard at:
{dashboard_url}

---
This is an automated message from {from_name}. Please do not reply to this email."#,
            to_name = to_name,
            document_title = document_title,
            dashboard_url = self.public_url,
            from_name = self.from_name
        );

        self.send_email(to_email, to_name, &subject, &html_body, &plain_body)
            .await
    }

    async fn send_email(
        &self,
        to_email: &str,
        to_name: &str,
        subject: &str,
        html_body: &str,
        _plain_body: &str,
    ) -> Result<()> {
        let from: Mailbox = format!("{} <{}>", self.from_name, self.from_email).parse()?;
        let to: Mailbox = format!("{} <{}>", to_name, to_email).parse()?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())?;

        match self.transport.send(email).await {
            Ok(_) => {
                info!("Email sent successfully to {}", to_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send email to {}: {}", to_email, e);
                Err(anyhow::anyhow!("Failed to send email: {}", e))
            }
        }
    }
}

pub fn create_email_service(config: &Config) -> Result<Option<EmailService>> {
    if config.smtp_host.is_empty() || config.smtp_host == "localhost" {
        info!("Email service not configured, emails will be logged but not sent");
        return Ok(None);
    }

    Ok(Some(EmailService::new(config)?))
}
