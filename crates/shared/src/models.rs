//! Shared data models for the portfolio application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use validator::Validate;

/// Blog post model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Option<Thing>,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub reading_time: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Blog post with rendered HTML content.
#[derive(Debug, Clone, Serialize)]
pub struct BlogPostRendered {
    pub id: Option<Thing>,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content_html: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub reading_time: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BlogPost {
    /// Convert to rendered version with HTML content.
    pub fn to_rendered(&self, html: String) -> BlogPostRendered {
        BlogPostRendered {
            id: self.id.clone(),
            title: self.title.clone(),
            slug: self.slug.clone(),
            excerpt: self.excerpt.clone(),
            content_html: html,
            tags: self.tags.clone(),
            published: self.published,
            reading_time: self.reading_time,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Create blog post request.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBlogPost {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1, max = 200))]
    pub slug: String,
    #[validate(length(max = 500))]
    pub excerpt: String,
    pub content: String,
    pub tags: Vec<String>,
    pub published: bool,
}

/// Project model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<Thing>,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub content: String,
    pub tech_stack: Vec<String>,
    pub url: Option<String>,
    pub github_url: Option<String>,
    pub image_url: Option<String>,
    pub featured: bool,
    pub order_index: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Project with rendered HTML content.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectRendered {
    pub id: Option<Thing>,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub content_html: String,
    pub tech_stack: Vec<String>,
    pub url: Option<String>,
    pub github_url: Option<String>,
    pub image_url: Option<String>,
    pub featured: bool,
    pub order_index: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Convert to rendered version with HTML content.
    pub fn to_rendered(&self, html: String) -> ProjectRendered {
        ProjectRendered {
            id: self.id.clone(),
            title: self.title.clone(),
            slug: self.slug.clone(),
            description: self.description.clone(),
            content_html: html,
            tech_stack: self.tech_stack.clone(),
            url: self.url.clone(),
            github_url: self.github_url.clone(),
            image_url: self.image_url.clone(),
            featured: self.featured,
            order_index: self.order_index,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Contact message model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactMessage {
    pub id: Option<Thing>,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// Contact form submission.
#[derive(Debug, Deserialize, Validate)]
pub struct ContactForm {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 200))]
    pub subject: String,
    #[validate(length(min = 10, max = 5000))]
    pub message: String,
    /// Honeypot field - should be empty for real submissions
    #[serde(default)]
    pub website: String,
}

/// Site setting model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub id: Option<Thing>,
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

/// Skill model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: Option<Thing>,
    pub name: String,
    pub category: String,
    pub proficiency: i32,
    pub icon: Option<String>,
    pub order_index: i32,
}

/// Social link model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    pub id: Option<Thing>,
    pub platform: String,
    pub url: String,
    pub icon: String,
    pub order_index: i32,
}

/// Experience model for work history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: Option<Thing>,
    pub title: String,
    pub company: String,
    pub period: String,
    pub description: String,
    pub order_index: i32,
}

/// Grouped skills by category.
#[derive(Debug, Clone, Serialize)]
pub struct SkillGroup {
    pub category: String,
    pub skills: Vec<Skill>,
}

/// Site metadata for templates.
#[derive(Debug, Clone, Serialize, Default)]
pub struct SiteMeta {
    pub title: String,
    pub description: String,
    pub author: String,
    pub url: String,
    pub portfolio: String,
}

impl SiteMeta {
    pub fn with_page_title(&self, page: &str) -> String {
        format!("{} | {}", page, self.title)
    }
}
