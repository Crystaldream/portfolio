use shared::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub site_url: String,
    pub discord_webhook_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            site_url: std::env::var("SITE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            discord_webhook_url: std::env::var("DISCORD_WEBHOOK_URL").ok(),
        })
    }
}
