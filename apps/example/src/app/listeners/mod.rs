//! Event listeners.
//!
//! Listeners are registered against the global [`EventDispatcher`] by the
//! [`EventServiceProvider`](crate::app::providers::event_service_provider).
//! Each listener is an async closure keyed to an event type.

use std::sync::Arc;
use lara_events::SharedDispatcher;
use lara_mail::Mailer;

use crate::app::events::{PasswordResetRequested, UserLoggedIn, UserRegistered};
use crate::app::mail::{PasswordResetEmail, WelcomeEmail};

/// Wire every listener to its event. Called once during boot.
pub async fn register(dispatcher: &SharedDispatcher) {
    register_send_welcome_email(dispatcher).await;
    register_log_user_login(dispatcher).await;
    register_send_password_reset(dispatcher).await;
}

/// On `UserRegistered` → send the welcome email.
async fn register_send_welcome_email(dispatcher: &SharedDispatcher) {
    dispatcher
        .listen::<UserRegistered, _, _>(|event: Arc<UserRegistered>| async move {
            tracing::info!(user_id = event.user_id, "Listener: SendWelcomeEmail");
            let mail = WelcomeEmail {
                name: event.name.clone(),
                email: event.email.clone(),
            };
            if let Err(e) = Mailer::send(mail).await {
                tracing::warn!("welcome email failed: {}", e);
            }
        })
        .await;
}

/// On `UserLoggedIn` → write an audit log entry.
async fn register_log_user_login(dispatcher: &SharedDispatcher) {
    dispatcher
        .listen::<UserLoggedIn, _, _>(|event: Arc<UserLoggedIn>| async move {
            tracing::info!(
                user_id = event.user_id,
                email = %event.email,
                ip = %event.ip,
                "Listener: LogUserLogin — user authenticated"
            );
        })
        .await;
}

/// On `PasswordResetRequested` → email the reset link.
async fn register_send_password_reset(dispatcher: &SharedDispatcher) {
    dispatcher
        .listen::<PasswordResetRequested, _, _>(|event: Arc<PasswordResetRequested>| async move {
            tracing::info!(user_id = event.user_id, "Listener: SendPasswordReset");
            let mail = PasswordResetEmail {
                name: event.email.clone(),
                email: event.email.clone(),
                reset_url: format!("https://example.com/reset-password?token={}", event.token),
                expires_in: 60,
            };
            if let Err(e) = Mailer::send(mail).await {
                tracing::warn!("password reset email failed: {}", e);
            }
        })
        .await;
}
