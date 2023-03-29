#[derive(Clone)]
pub struct Config {
    pub organization_id: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            organization_id: std::env::var("ORGANIZATION_ID")?,
        });
    }
}
