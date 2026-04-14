use crate::{
    config::GLOBAL_CONFIG,
    dtos::smtp::{CodeEmailContext, Notification},
};
use anyhow::{Error, Ok};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use tera::{Context, Tera};

fn smtp_disabled() -> bool {
    std::env::var("DISABLE_SMTP")
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false)
}

pub async fn send_code_to_update_password(code: String, email: String) -> Result<(), Error> {
    if smtp_disabled() {
        tracing::debug!("SMTP disabled, skipping password reset email to {}", email);
        return Ok(());
    }

    let subject = "Код для изменения пароля".to_string();
    let code_email_context = CodeEmailContext {
        code,
        email: email.clone(),
        subject: subject.clone(),
        text: "Вы изменяете пароль на вашем аккаунте. Необходимо ввести код для подтверждения изменения".to_string()
    };

    let tera = Tera::new("api/templates/**/*")?;
    let mut ctx = Context::new();
    ctx.insert("code_email_context", &code_email_context);
    let html = tera.render("verification_code.html", &ctx)?;

    send_message_to_email(email, html, subject).await?;

    Ok(())
}
pub async fn send_code_to_update_email(code: String, email: String) -> Result<(), Error> {
    if smtp_disabled() {
        tracing::debug!("SMTP disabled, skipping email change code to {}", email);
        return Ok(());
    }

    let subject = "Код для изменения почты".to_string();
    let code_email_context = CodeEmailContext {
        code,
        email: email.clone(),
        subject: subject.clone(),
        text: "Вы изменяете почту на вашем аккаунте. Необходимо ввести код для изменения почты для потверждения изменения".to_string()
    };

    let tera = Tera::new("api/templates/**/*")?;
    let mut ctx = Context::new();
    ctx.insert("code_email_context", &code_email_context);
    let html = tera.render("verification_code.html", &ctx)?;

    send_message_to_email(email, html, subject).await?;

    Ok(())
}
pub async fn send_invitation(
    id: String,
    first_name: String,
    last_name: String,
    email: String,
) -> Result<(), Error> {
    if smtp_disabled() {
        tracing::debug!("SMTP disabled, skipping invitation email to {}", email);
        return Ok(());
    }

    let subject = "Приглашение на регистрацию".to_string();
    let link = format!("{}/auth/registration?code={}", GLOBAL_CONFIG.client_url, id);
    let invitation_text = format!(
        "Вас пригласил(-а) зарегистрироваться на портал HITS {} {} \
        в качестве пользователя. Для регистрации на сервисе \
        перейдите по данной ссылке и заполните все поля.",
        first_name, last_name
    );

    let notification = Notification {
        email: email.clone(),
        title: subject.clone(),
        message: invitation_text,
        link,
        button_name: "Зарегистрироваться".to_string(),
    };

    let tera = Tera::new("api/templates/**/*")?;
    let mut ctx = Context::new();
    ctx.insert("notification", &notification);
    let html = tera.render("notification.html", &ctx)?;

    send_message_to_email(email, html, subject).await?;

    Ok(())
}
pub async fn send_team_invitation(
    team_id: String,
    team_name: String,
    first_name: String,
    last_name: String,
    email: String,
) -> Result<(), Error> {
    if smtp_disabled() {
        tracing::debug!("SMTP disabled, skipping team invitation email to {}", email);
        return Ok(());
    }

    let subject = "Приглашение в команду".to_string();
    let link = format!("{}/team/list/{}", GLOBAL_CONFIG.client_url, team_id);
    let invitation_text = format!(
        "Вас пригласил(-а) {} {} в команду \"{}\" в качестве участника.",
        first_name, last_name, team_name
    );

    let notification = Notification {
        email: email.clone(),
        title: subject.clone(),
        message: invitation_text,
        link,
        button_name: "Перейти в команду".to_string(),
    };

    let tera = Tera::new("api/templates/**/*")?;
    let mut ctx = Context::new();
    ctx.insert("notification", &notification);
    let html = tera.render("notification.html", &ctx)?;

    send_message_to_email(email, html, subject).await?;

    Ok(())
}
pub async fn send_message_to_email(
    email: String,
    html: String,
    subject: String,
) -> Result<(), Error> {
    if smtp_disabled() {
        tracing::debug!("SMTP disabled, skipping email {} to {}", subject, email);
        return Ok(());
    }

    let mailer: AsyncSmtpTransport<Tokio1Executor> = if cfg!(debug_assertions) {
        let creds = Credentials::new(
            GLOBAL_CONFIG.smtp_user.to_owned(),
            GLOBAL_CONFIG.smtp_password.to_owned(),
        );

        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&GLOBAL_CONFIG.smtp_host)?
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&GLOBAL_CONFIG.smtp_host).build()
    };

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
