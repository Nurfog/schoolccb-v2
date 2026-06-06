use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_address: String,
    pub from_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3005".into())
                .parse()
                .expect("PORT must be a valid number"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET debe estar definida en producción"),
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".into()),
            smtp_port: env::var("SMTP_PORT").unwrap_or_else(|_| "587".into()).parse().unwrap_or(587),
            smtp_user: env::var("SMTP_USER").unwrap_or_default(),
            smtp_pass: env::var("SMTP_PASS").unwrap_or_default(),
            from_address: env::var("FROM_ADDRESS").unwrap_or_else(|_| "noreply@schoolccb.cl".into()),
            from_name: env::var("FROM_NAME").unwrap_or_else(|_| "SchoolCBB".into()),
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
