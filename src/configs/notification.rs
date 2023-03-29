pub struct Config {
    pub email: String,
    pub password: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            email: std::env::var("NOTIFICATION_EMAIL")?,
            password: std::env::var("NOTIFICATION_PASSWORD")?,
        });
    }
}
