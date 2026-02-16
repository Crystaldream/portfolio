//! Page routes - serve full HTML pages.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;

use shared::models::{BlogPost, BlogPostRendered, Experience, Project, ProjectRendered, Setting, Skill, SkillGroup, SocialLink};

use crate::css;
use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::templates::{
    AboutTemplate, BlogPostTemplate, BlogTemplate, ContactTemplate, HomeTemplate, NotFoundTemplate,
    ProjectTemplate, ProjectsTemplate, HtmlTemplate,
};

/// Create the pages router.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(home))
        .route("/about", get(about))
        .route("/projects", get(projects))
        .route("/projects/{slug}", get(project_detail))
        .route("/blog", get(blog))
        .route("/blog/{slug}", get(blog_post))
        .route("/contact", get(contact))
        .fallback(not_found)
}

/// Home page handler.
async fn home(State(state): State<Arc<AppState>>) -> Result<HtmlTemplate<HomeTemplate>> {
    // Fetch featured projects
    let featured_projects: Vec<Project> = state
        .db
        .client()
        .query("SELECT * FROM projects WHERE featured = true ORDER BY order_index ASC LIMIT 3")
        .await?
        .take(0)?;

    let featured_projects: Vec<ProjectRendered> = featured_projects
        .iter()
        .map(|p| p.to_rendered(state.markdown.render(&p.content)))
        .collect();

    // Fetch recent blog posts
    let recent_posts: Vec<BlogPost> = state
        .db
        .client()
        .query("SELECT * FROM posts WHERE published = true ORDER BY created_at DESC LIMIT 3")
        .await?
        .take(0)?;

    let recent_posts: Vec<BlogPostRendered> = recent_posts
        .iter()
        .map(|p| p.to_rendered(state.markdown.render(&p.content)))
        .collect();

    // Fetch social links
    let social_links: Vec<SocialLink> = state
        .db
        .client()
        .query("SELECT * FROM social_links ORDER BY order_index ASC")
        .await?
        .take(0)?;

    // Fetch intro text
    let intro: Option<Setting> = state
        .db
        .client()
        .query("SELECT * FROM settings WHERE key = 'about_intro' LIMIT 1")
        .await?
        .take(0)?;

    let intro_text = intro.map(|s| s.value).unwrap_or_default();

    Ok(HtmlTemplate(HomeTemplate {
        site: state.site_meta.clone(),
        featured_projects,
        recent_posts,
        social_links,
        intro_text,
        css: css::with_base(css::HOME),
    }))
}

/// About page handler.
async fn about(State(state): State<Arc<AppState>>) -> Result<HtmlTemplate<AboutTemplate>> {
    // Fetch about content
    let about_content: Option<Setting> = state
        .db
        .client()
        .query("SELECT * FROM settings WHERE key = 'about_content' LIMIT 1")
        .await?
        .take(0)?;

    let content_html = about_content
        .map(|s| state.markdown.render(&s.value))
        .unwrap_or_default();

    // Fetch skills grouped by category
    let skills: Vec<Skill> = state
        .db
        .client()
        .query("SELECT * FROM skills ORDER BY category ASC, order_index ASC")
        .await?
        .take(0)?;

    // Group skills by category
    let mut skill_groups: Vec<SkillGroup> = Vec::new();
    let mut current_category = String::new();

    for skill in skills {
        if skill.category != current_category {
            current_category = skill.category.clone();
            skill_groups.push(SkillGroup {
                category: current_category.clone(),
                skills: vec![skill],
            });
        } else if let Some(group) = skill_groups.last_mut() {
            group.skills.push(skill);
        }
    }

    // Fetch social links
    let social_links: Vec<SocialLink> = state
        .db
        .client()
        .query("SELECT * FROM social_links ORDER BY order_index ASC")
        .await?
        .take(0)?;

    // Fetch experiences
    let experiences: Vec<Experience> = state
        .db
        .client()
        .query("SELECT * FROM experiences ORDER BY order_index ASC")
        .await?
        .take(0)?;

    Ok(HtmlTemplate(AboutTemplate {
        site: state.site_meta.clone(),
        content_html,
        skill_groups,
        social_links,
        experiences,
        css: css::with_base(css::ABOUT),
    }))
}

/// Projects page handler.
async fn projects(State(state): State<Arc<AppState>>) -> Result<HtmlTemplate<ProjectsTemplate>> {
    let projects: Vec<Project> = state
        .db
        .client()
        .query("SELECT * FROM projects ORDER BY order_index ASC")
        .await?
        .take(0)?;

    let projects: Vec<ProjectRendered> = projects
        .iter()
        .map(|p| p.to_rendered(state.markdown.render(&p.content)))
        .collect();

    Ok(HtmlTemplate(ProjectsTemplate {
        site: state.site_meta.clone(),
        projects,
        css: css::combine(&[css::HOME, css::PROJECTS]),
    }))
}

/// Project detail page handler.
async fn project_detail(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<HtmlTemplate<ProjectTemplate>> {
    let mut result = state
        .db
        .client()
        .query("SELECT * FROM projects WHERE slug = $slug LIMIT 1")
        .bind(("slug", slug.clone()))
        .await?;

    let project: Option<Project> = result.take(0)?;

    let project = project.ok_or_else(|| AppError::not_found(format!("Project '{}' not found", slug)))?;
    let project = project.to_rendered(state.markdown.render(&project.content));

    Ok(HtmlTemplate(ProjectTemplate {
        site: state.site_meta.clone(),
        project,
        css: css::combine(&[css::HOME, css::PROJECTS]),
    }))
}

/// Blog page handler.
async fn blog(State(state): State<Arc<AppState>>) -> Result<HtmlTemplate<BlogTemplate>> {
    let posts: Vec<BlogPost> = state
        .db
        .client()
        .query("SELECT * FROM posts WHERE published = true ORDER BY created_at DESC")
        .await?
        .take(0)?;

    let posts: Vec<BlogPostRendered> = posts
        .iter()
        .map(|p| p.to_rendered(state.markdown.render(&p.content)))
        .collect();

    // Collect all unique tags
    let mut all_tags: Vec<String> = posts.iter().flat_map(|p| p.tags.clone()).collect();
    all_tags.sort();
    all_tags.dedup();

    Ok(HtmlTemplate(BlogTemplate {
        site: state.site_meta.clone(),
        posts,
        tags: all_tags,
        css: css::with_base(css::BLOG),
    }))
}

/// Blog post detail page handler.
async fn blog_post(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<HtmlTemplate<BlogPostTemplate>> {
    let mut result = state
        .db
        .client()
        .query("SELECT * FROM posts WHERE slug = $slug AND published = true LIMIT 1")
        .bind(("slug", slug.clone()))
        .await?;

    let post: Option<BlogPost> = result.take(0)?;

    let post = post.ok_or_else(|| AppError::not_found(format!("Blog post '{}' not found", slug)))?;
    let post = post.to_rendered(state.markdown.render(&post.content));

    Ok(HtmlTemplate(BlogPostTemplate {
        site: state.site_meta.clone(),
        post,
        css: css::with_base(css::BLOG),
    }))
}

/// Contact page handler.
async fn contact(State(state): State<Arc<AppState>>) -> Result<HtmlTemplate<ContactTemplate>> {
    let social_links: Vec<SocialLink> = state
        .db
        .client()
        .query("SELECT * FROM social_links ORDER BY order_index ASC")
        .await?
        .take(0)?;

    Ok(HtmlTemplate(ContactTemplate {
        site: state.site_meta.clone(),
        social_links,
        css: css::with_base(css::CONTACT),
    }))
}

/// 404 Not Found handler.
async fn not_found(State(state): State<Arc<AppState>>) -> HtmlTemplate<NotFoundTemplate> {
    HtmlTemplate(NotFoundTemplate {
        site: state.site_meta.clone(),
        css: css::with_base(css::ERROR_404),
    })
}
