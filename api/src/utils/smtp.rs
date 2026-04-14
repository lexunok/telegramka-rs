use crate::{config::GLOBAL_CONFIG, dtos::smtp::CodeEmailContext};
use anyhow::{Error, Ok};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use tera::{Context, Tera};

pub async fn send_auth_code(code: String, email: String) -> Result<(), Error> {
    let subject = "Код подтверждения".to_string();
    let text = "Введите код для продолжения операции в Telegramka.".to_string();
    let code_email_context = CodeEmailContext {
        code,
        email,
        subject: subject.clone(),
        text,
    };

    let tera = Tera::new("api/templates/**/*")?;
    let mut ctx = Context::new();
    ctx.insert("code_email_context", &code_email_context);
    let html = tera.render("verification_code.html", &ctx)?;

    send_message_to_email(code_email_context.email, html, subject).await?;

    Ok(())
}
pub async fn send_message_to_email(
    email: String,
    html: String,
    subject: String,
) -> Result<(), Error> {
    let creds = Credentials::new(
        GLOBAL_CONFIG.smtp_user.to_owned(),
        GLOBAL_CONFIG.smtp_password.to_owned(),
    );

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&GLOBAL_CONFIG.smtp_host)?
            .credentials(creds)
            .build();

    let message = Message::builder()
        .from(GLOBAL_CONFIG.smtp_from.parse().unwrap())
        .to(email.parse().unwrap())
        .subject(subject.clone())
        .header(ContentType::TEXT_HTML)
        .body(html)?;

    mailer.send(message).await?;

    tracing::debug!("Отправляем письмо {} на {}", subject, email,);

    Ok(())
}
