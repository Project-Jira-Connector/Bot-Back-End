pub struct Config {
    pub address: String,
    pub port: u16,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            address: std::env::var("BIND_ADDRESS")?,
            port: std::env::var("BIND_PORT")?.parse()?,
        });
    }
}
