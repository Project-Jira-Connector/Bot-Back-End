pub struct Config {
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            username: std::env::var("MONGODB_USERNAME")?,
            password: std::env::var("MONGODB_PASSWORD")?,
        });
    }
}
