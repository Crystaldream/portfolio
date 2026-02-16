//! Askama templates for HTML rendering.

use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use shared::models::{
    BlogPostRendered, Experience, ProjectRendered, SkillGroup, SiteMeta, SocialLink,
};

/// Wrapper to convert Askama templates to Axum responses
pub struct HtmlTemplate<T>(pub T);

impl<T: Template> IntoResponse for HtmlTemplate<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", err),
            )
                .into_response(),
        }
    }
}

// =============================================================================
// Full Page Templates
// =============================================================================

/// Home page template.
#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub site: SiteMeta,
    pub featured_projects: Vec<ProjectRendered>,
    pub recent_posts: Vec<BlogPostRendered>,
    pub social_links: Vec<SocialLink>,
    pub intro_text: String,
    pub css: String,
}

/// About page template.
#[derive(Template)]
#[template(path = "pages/about.html")]
pub struct AboutTemplate {
    pub site: SiteMeta,
    pub content_html: String,
    pub skill_groups: Vec<SkillGroup>,
    pub social_links: Vec<SocialLink>,
    pub experiences: Vec<Experience>,
    pub css: String,
}

/// Projects listing page template.
#[derive(Template)]
#[template(path = "pages/projects.html")]
pub struct ProjectsTemplate {
    pub site: SiteMeta,
    pub projects: Vec<ProjectRendered>,
    pub css: String,
}

/// Single project page template.
#[derive(Template)]
#[template(path = "pages/project.html")]
pub struct ProjectTemplate {
    pub site: SiteMeta,
    pub project: ProjectRendered,
    pub css: String,
}

/// Blog listing page template.
#[derive(Template)]
#[template(path = "pages/blog.html")]
pub struct BlogTemplate {
    pub site: SiteMeta,
    pub posts: Vec<BlogPostRendered>,
    pub tags: Vec<String>,
    pub css: String,
}

/// Single blog post page template.
#[derive(Template)]
#[template(path = "pages/post.html")]
pub struct BlogPostTemplate {
    pub site: SiteMeta,
    pub post: BlogPostRendered,
    pub css: String,
}

/// Contact page template.
#[derive(Template)]
#[template(path = "pages/contact.html")]
pub struct ContactTemplate {
    pub site: SiteMeta,
    pub social_links: Vec<SocialLink>,
    pub css: String,
}

/// 404 Not Found page template.
#[derive(Template)]
#[template(path = "pages/404.html")]
pub struct NotFoundTemplate {
    pub site: SiteMeta,
    pub css: String,
}

// =============================================================================
// Partial Templates (for HTMX)
// =============================================================================

/// Blog posts list partial.
#[derive(Template)]
#[template(path = "partials/blog_list.html")]
pub struct BlogListPartial {
    pub posts: Vec<BlogPostRendered>,
}
