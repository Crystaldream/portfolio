//! Database connection and utilities for SurrealDB.

use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{info, instrument};

use crate::error::Result;

#[derive(Clone)]
pub struct Database {
    client: Surreal<Any>,
}

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl DbConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: std::env::var("SURREAL_URL").unwrap_or_else(|_| "ws://127.0.0.1:8000".to_string()),
            username: std::env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string()),
            namespace: std::env::var("SURREAL_NS").unwrap_or_else(|_| "portfolio".to_string()),
            database: std::env::var("SURREAL_DB").unwrap_or_else(|_| "main".to_string()),
        })
    }
}

impl Database {
    #[instrument(skip(config), fields(url = %config.url, ns = %config.namespace, db = %config.database))]
    pub async fn connect(config: &DbConfig) -> Result<Self> {
        info!("Connecting to SurrealDB...");

        let client = connect(&config.url).await?;

        info!("Connected, signing in...");

        client
            .signin(Root {
                username: &config.username,
                password: &config.password,
            })
            .await?;

        info!("Signed in, selecting namespace/database...");

        client
            .use_ns(&config.namespace)
            .use_db(&config.database)
            .await?;

        info!("Connected to SurrealDB successfully");

        Ok(Self { client })
    }

    pub fn client(&self) -> &Surreal<Any> {
        &self.client
    }

    #[instrument(skip(self))]
    pub async fn init_schema(&self) -> Result<()> {
        info!("Initializing database schema...");

        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS posts SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS title ON TABLE posts TYPE string;
                DEFINE FIELD IF NOT EXISTS slug ON TABLE posts TYPE string;
                DEFINE FIELD IF NOT EXISTS excerpt ON TABLE posts TYPE string;
                DEFINE FIELD IF NOT EXISTS content ON TABLE posts TYPE string;
                DEFINE FIELD IF NOT EXISTS tags ON TABLE posts TYPE array;
                DEFINE FIELD IF NOT EXISTS published ON TABLE posts TYPE bool DEFAULT false;
                DEFINE FIELD IF NOT EXISTS reading_time ON TABLE posts TYPE int;
                DEFINE FIELD IF NOT EXISTS created_at ON TABLE posts TYPE datetime DEFAULT time::now();
                DEFINE FIELD IF NOT EXISTS updated_at ON TABLE posts TYPE datetime DEFAULT time::now();
                DEFINE INDEX IF NOT EXISTS idx_posts_slug ON TABLE posts COLUMNS slug UNIQUE;
                "#,
            )
            .await?;

        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS projects SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS title ON TABLE projects TYPE string;
                DEFINE FIELD IF NOT EXISTS slug ON TABLE projects TYPE string;
                DEFINE FIELD IF NOT EXISTS description ON TABLE projects TYPE string;
                DEFINE FIELD IF NOT EXISTS content ON TABLE projects TYPE string;
                DEFINE FIELD IF NOT EXISTS tech_stack ON TABLE projects TYPE array;
                DEFINE FIELD IF NOT EXISTS url ON TABLE projects TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS github_url ON TABLE projects TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS image_url ON TABLE projects TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS featured ON TABLE projects TYPE bool DEFAULT false;
                DEFINE FIELD IF NOT EXISTS order_index ON TABLE projects TYPE int DEFAULT 0;
                DEFINE FIELD IF NOT EXISTS created_at ON TABLE projects TYPE datetime DEFAULT time::now();
                DEFINE FIELD IF NOT EXISTS updated_at ON TABLE projects TYPE datetime DEFAULT time::now();
                DEFINE INDEX IF NOT EXISTS idx_projects_slug ON TABLE projects COLUMNS slug UNIQUE;
                "#,
            )
            .await?;

        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS contacts SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS name ON TABLE contacts TYPE string;
                DEFINE FIELD IF NOT EXISTS email ON TABLE contacts TYPE string;
                DEFINE FIELD IF NOT EXISTS subject ON TABLE contacts TYPE string;
                DEFINE FIELD IF NOT EXISTS message ON TABLE contacts TYPE string;
                DEFINE FIELD IF NOT EXISTS read ON TABLE contacts TYPE bool DEFAULT false;
                DEFINE FIELD IF NOT EXISTS created_at ON TABLE contacts TYPE datetime DEFAULT time::now();
                "#,
            )
            .await?;

        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS settings SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS key ON TABLE settings TYPE string;
                DEFINE FIELD IF NOT EXISTS value ON TABLE settings TYPE string;
                DEFINE FIELD IF NOT EXISTS updated_at ON TABLE settings TYPE datetime DEFAULT time::now();
                DEFINE INDEX IF NOT EXISTS idx_settings_key ON TABLE settings COLUMNS key UNIQUE;
                "#,
            )
            .await?;

        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS skills SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS name ON TABLE skills TYPE string;
                DEFINE FIELD IF NOT EXISTS category ON TABLE skills TYPE string;
                DEFINE FIELD IF NOT EXISTS proficiency ON TABLE skills TYPE int;
                DEFINE FIELD IF NOT EXISTS icon ON TABLE skills TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS order_index ON TABLE skills TYPE int DEFAULT 0;
                "#,
            )
            .await?;

        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS social_links SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS platform ON TABLE social_links TYPE string;
                DEFINE FIELD IF NOT EXISTS url ON TABLE social_links TYPE string;
                DEFINE FIELD IF NOT EXISTS icon ON TABLE social_links TYPE string;
                DEFINE FIELD IF NOT EXISTS order_index ON TABLE social_links TYPE int DEFAULT 0;
                DEFINE INDEX IF NOT EXISTS idx_social_platform ON TABLE social_links COLUMNS platform UNIQUE;
                "#,
            )
            .await?;

        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS experiences SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS title ON TABLE experiences TYPE string;
                DEFINE FIELD IF NOT EXISTS company ON TABLE experiences TYPE string;
                DEFINE FIELD IF NOT EXISTS period ON TABLE experiences TYPE string;
                DEFINE FIELD IF NOT EXISTS description ON TABLE experiences TYPE string;
                DEFINE FIELD IF NOT EXISTS order_index ON TABLE experiences TYPE int DEFAULT 0;
                "#,
            )
            .await?;

        info!("Database schema initialized");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn seed_dev_data(&self) -> Result<()> {
        info!("Checking for existing data...");

        let posts: Vec<crate::models::BlogPost> =
            self.client.select("posts").await.unwrap_or_default();

        if !posts.is_empty() {
            info!("Data already exists, skipping seed");
            return Ok(());
        }

        info!("Seeding development data...");

        self.client
            .query(
                r#"
                INSERT INTO social_links (platform, url, icon, order_index) VALUES
                    ('github', 'https://github.com/yourusername', 'github', 1),
                    ('linkedin', 'https://linkedin.com/in/yourusername', 'linkedin', 2);
                "#,
            )
            .await?;

        self.client
            .query(
                r#"
                INSERT INTO settings (key, value) VALUES
                    ('site_title', 'Your Name'),
                    ('site_description', 'Software Developer'),
                    ('about_intro', 'Hello! I am a passionate software developer.'),
                    ('about_content', '## About Me\n\nI build things.');
                "#,
            )
            .await?;

        self.client
            .query(
                r#"
                INSERT INTO experiences (title, company, period, description, order_index) VALUES
                    ('Senior Software Developer', 'Example Company', '2022 - Present', 'Leading development of web applications using Rust and modern frontend technologies.', 1),
                    ('Software Developer', 'Previous Company', '2020 - 2022', 'Developed and maintained full-stack applications.', 2);
                "#,
            )
            .await?;

        info!("Development data seeded");
        Ok(())
    }
}
