pub struct Config {
    pub schedule: cron::Schedule,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            schedule: std::env::var("SCHEDULE")?.parse()?,
        });
    }
}
