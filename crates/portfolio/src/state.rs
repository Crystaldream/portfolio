use shared::db::Database;
use shared::markdown::MarkdownRenderer;
use shared::models::SiteMeta;

use crate::config::Config;

pub struct AppState {
    pub db: Database,
    pub config: Config,
    pub markdown: MarkdownRenderer,
    pub site_meta: SiteMeta,
}

impl AppState {
    pub fn new(db: Database, config: Config) -> Self {
        let site_meta = SiteMeta {
            author: "Telmo Reinas".to_string(),
            title: "Software Engineer".to_string(),
            portfolio: "'s Portfolio".to_string(),
            url: config.site_url.clone(),
            description: String::new(),
        };

        Self {
            db,
            config,
            markdown: MarkdownRenderer::new(),
            site_meta,
        }
    }
}
