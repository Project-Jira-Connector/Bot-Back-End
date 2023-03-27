pub struct Config {
    pub key: String,
    pub secret: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            key: std::env::var("AWS_ACCESS_KEY")?,
            secret: std::env::var("AWS_SECRET_KEY")?,
        });
    }
}
