use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub service_fee_bps: i64,
    pub reservation_minutes: i64,
    pub database_url: String,
    pub mongodb_uri: String,
    pub mongodb_database: String,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub web_success_url: String,
    pub web_cancel_url: String,
    pub allowed_origin: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_values(std::env::vars())
    }

    pub fn from_values<K, V>(values: impl IntoIterator<Item = (K, V)>) -> Result<Self, String>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let values: HashMap<String, String> = values
            .into_iter()
            .map(|(key, value)| (key.as_ref().to_owned(), value.as_ref().to_owned()))
            .collect();
        let required = |name: &str| {
            values
                .get(name)
                .filter(|value| !value.is_empty())
                .cloned()
                .ok_or_else(|| format!("{name} is required"))
        };
        let parse = |name: &str, default: &str| {
            values
                .get(name)
                .map(String::as_str)
                .unwrap_or(default)
                .parse::<i64>()
                .map_err(|_| format!("{name} must be an integer"))
        };

        let port_value = parse("PORT", "8080")?;
        let port = u16::try_from(port_value).map_err(|_| "PORT must be a u16".to_owned())?;
        let service_fee_bps = parse("SERVICE_FEE_BPS", "600")?;
        let reservation_minutes = parse("RESERVATION_MINUTES", "30")?;
        if !(0..=10_000).contains(&service_fee_bps) {
            return Err("SERVICE_FEE_BPS must be between 0 and 10000".into());
        }
        if reservation_minutes <= 0 {
            return Err("RESERVATION_MINUTES must be positive".into());
        }
        let stripe_secret_key = required("STRIPE_SECRET_KEY")?;
        if !stripe_secret_key.starts_with("sk_test_") {
            return Err("STRIPE_SECRET_KEY must be a Stripe test-mode key".into());
        }

        let mongodb_database = required("MONGODB_DATABASE")?;
        if mongodb_database
            .chars()
            .any(|character| matches!(character, '/' | '\\' | '.' | ' ' | '"' | '$' | '\0'))
        {
            return Err("MONGODB_DATABASE contains unsupported characters".into());
        }

        Ok(Self {
            port,
            service_fee_bps,
            reservation_minutes,
            database_url: required("DATABASE_URL")?,
            mongodb_uri: required("MONGODB_URI")?,
            mongodb_database,
            supabase_url: required("SUPABASE_URL")?,
            supabase_anon_key: required("SUPABASE_ANON_KEY")?,
            stripe_secret_key,
            stripe_webhook_secret: required("STRIPE_WEBHOOK_SECRET")?,
            web_success_url: required("WEB_SUCCESS_URL")?,
            web_cancel_url: required("WEB_CANCEL_URL")?,
            allowed_origin: required("ALLOWED_ORIGIN")?,
        })
    }
}
