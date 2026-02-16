//! Compile-time embedded CSS to eliminate render-blocking requests.

pub const BASE: &str = include_str!("../static/css/base.css");
pub const HOME: &str = include_str!("../static/css/home.css");
pub const PROJECTS: &str = include_str!("../static/css/projects.css");
pub const BLOG: &str = include_str!("../static/css/blog.css");
pub const ABOUT: &str = include_str!("../static/css/about.css");
pub const CONTACT: &str = include_str!("../static/css/contact.css");
pub const ERROR_404: &str = include_str!("../static/css/404.css");

/// Concatenate base CSS with page-specific CSS.
pub fn with_base(page_css: &str) -> String {
    format!("{}\n{}", BASE, page_css)
}

/// For pages needing multiple CSS files (e.g., projects + home).
pub fn combine(css_files: &[&str]) -> String {
    let mut result = BASE.to_string();
    for css in css_files {
        result.push('\n');
        result.push_str(css);
    }
    result
}
