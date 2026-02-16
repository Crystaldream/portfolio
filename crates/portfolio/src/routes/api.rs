use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use validator::Validate;

use shared::models::{BlogPost, ContactForm, ContactMessage, Project};

use crate::error::{AppError, Result};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/posts", get(list_posts))
        .route("/projects", get(list_projects))
        .route("/contact", post(submit_contact))
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }
}

async fn list_posts(State(state): State<Arc<AppState>>) -> Result<Json<ApiResponse<Vec<BlogPost>>>> {
    let posts: Vec<BlogPost> = state
        .db
        .client()
        .query("SELECT * FROM posts WHERE published = true ORDER BY created_at DESC")
        .await?
        .take(0)?;

    Ok(Json(ApiResponse::success(posts)))
}

async fn list_projects(State(state): State<Arc<AppState>>) -> Result<Json<ApiResponse<Vec<Project>>>> {
    let projects: Vec<Project> = state
        .db
        .client()
        .query("SELECT * FROM projects ORDER BY order_index ASC")
        .await?
        .take(0)?;

    Ok(Json(ApiResponse::success(projects)))
}

async fn submit_contact(
    State(state): State<Arc<AppState>>,
    Json(form): Json<ContactForm>,
) -> Result<Json<ApiResponse<&'static str>>> {
    form.validate().map_err(|e| AppError::validation(e.to_string()))?;

    let _: Option<ContactMessage> = state
        .db
        .client()
        .query(
            r#"
            INSERT INTO contacts (name, email, subject, message)
            VALUES ($name, $email, $subject, $message)
            "#,
        )
        .bind(("name", form.name.clone()))
        .bind(("email", form.email.clone()))
        .bind(("subject", form.subject.clone()))
        .bind(("message", form.message.clone()))
        .await?
        .take(0)?;

    if let Some(webhook_url) = &state.config.discord_webhook_url {
        send_discord_notification(webhook_url, &form).await.ok();
    }

    Ok(Json(ApiResponse::success("Message sent successfully")))
}

async fn send_discord_notification(webhook_url: &str, form: &ContactForm) -> Result<()> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "embeds": [{
            "title": format!("New Contact: {}", form.subject),
            "color": 0x89b4fa,
            "fields": [
                {"name": "Name", "value": &form.name, "inline": true},
                {"name": "Email", "value": &form.email, "inline": true},
                {"name": "Message", "value": &form.message}
            ],
            "timestamp": chrono::Utc::now().to_rfc3339()
        }]
    });

    client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::external(e.to_string()))?;

    Ok(())
}
