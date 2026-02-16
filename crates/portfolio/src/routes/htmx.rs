use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Html;
use axum::routing::{get, post};
use axum::Form;
use axum::Router;
use serde::Deserialize;
use validator::Validate;

use shared::models::{BlogPost, BlogPostRendered, ContactForm, ContactMessage};

use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::templates::{BlogListPartial, HtmlTemplate};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/posts", get(posts_list))
        .route("/posts/filter", get(posts_filter))
        .route("/contact/submit", post(contact_submit))
}

#[derive(Debug, Deserialize)]
pub struct PostsFilterQuery {
    pub tag: Option<String>,
}

async fn posts_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PostsFilterQuery>,
) -> Result<HtmlTemplate<BlogListPartial>> {
    let posts: Vec<BlogPost> = if let Some(ref tag) = query.tag {
        let mut result = state
            .db
            .client()
            .query("SELECT * FROM posts WHERE published = true AND $tag IN tags ORDER BY created_at DESC")
            .bind(("tag", tag.clone()))
            .await?;
        result.take(0)?
    } else {
        state
            .db
            .client()
            .query("SELECT * FROM posts WHERE published = true ORDER BY created_at DESC")
            .await?
            .take(0)?
    };

    let posts: Vec<BlogPostRendered> = posts
        .iter()
        .map(|p| p.to_rendered(state.markdown.render(&p.content)))
        .collect();

    Ok(HtmlTemplate(BlogListPartial { posts }))
}

async fn posts_filter(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PostsFilterQuery>,
) -> Result<HtmlTemplate<BlogListPartial>> {
    posts_list(State(state), Query(query)).await
}

async fn contact_submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ContactForm>,
) -> Result<Html<String>> {

    let spam_keywords = ["viagra", "casino", "crypto", "dating for sex", "lottery", "prize"];
    let message_lower = form.message.to_lowercase();
    let subject_lower = form.subject.to_lowercase();
    let link_count = form.message.matches("http").count();

    if !form.website.is_empty() 
        || spam_keywords.iter().any(|kw| message_lower.contains(kw) || subject_lower.contains(kw)) 
        || link_count > 0 {
        return Ok(Html(
            r#"<div class="form-success" role="status">
                <i class="fa-solid fa-circle-check icon" aria-hidden="true"></i>
                <h3>Message Sent!</h3>
                <p>Thank you for reaching out.</p>
            </div>"#.to_string()
        ));
    }

    if let Err(e) = form.validate() {
        return Ok(Html(format!(
            r#"<div class="form-error" role="alert">
                <p>Please fix the following errors:</p>
                <ul>{}</ul>
            </div>"#,
            e.field_errors()
                .iter()
                .flat_map(|(field, errors)| {
                    errors
                        .iter()
                        .map(move |e| format!("<li><strong>{}:</strong> {}</li>", field, e.message.as_deref().unwrap_or("Invalid")))
                })
                .collect::<Vec<_>>()
                .join("")
        )));
    }

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

    Ok(Html(
        r#"<div class="form-success" role="status">
            <i class="fa-solid fa-circle-check icon" aria-hidden="true"></i>
            <h3>Message Sent!</h3>
            <p>Thank you for reaching out. I'll get back to you soon.</p>
        </div>"#
            .to_string(),
    ))
}

async fn send_discord_notification(webhook_url: &str, form: &ContactForm) -> Result<()> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "embeds": [{
            "title": format!("New Contact with Subject: {}", form.subject),
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
