#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Server {
    pub address: String,
    pub port: u16,
}

impl Server {
    pub fn new() -> Self {
        return Self {
            address: std::env::var("BIND_ADDRESS").expect("BIND_ADDRESS must be defined"),
            port: std::env::var("BIND_PORT")
                .expect("BIND_PORT must be defined")
                .parse()
                .expect("BIND_PORT must be a valid port"),
        };
    }
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Database {
    pub username: String,
    pub password: String,
}

impl Database {
    pub fn new() -> Self {
        return Self {
            username: std::env::var("MONGODB_USERNAME").expect("MONGODB_USERNAME must be defined"),
            password: std::env::var("MONGODB_PASSWORD").expect("MONGODB_PASSWORD must be defined"),
        };
    }
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Notification {
    pub email: String,
    pub password: String,
}

impl Notification {
    pub fn new() -> Self {
        return Self {
            email: std::env::var("NOTIFICATION_EMAIL").expect("NOTIFICATION_EMAIL must be defined"),
            password: std::env::var("NOTIFICATION_PASSWORD")
                .expect("NOTIFICATION_PASSWORD must be defined"),
        };
    }
}

pub struct Storage {
    pub access_key: String,
    pub secret_key: String,
}

impl Storage {
    pub fn new() -> Self {
        return Self {
            access_key: std::env::var("AWS_ACCESS_KEY").expect("AWS_ACCESS_KEY must be defined"),
            secret_key: std::env::var("AWS_SECRET_KEY").expect("AWS_SECRET_KEY must be defined"),
        };
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Environment {
    pub server: Server,
    pub database: Database,
    pub notification: Notification,
    pub schedule: cron::Schedule,
    pub storage: Storage,
}

impl Environment {
    pub fn new() -> Self {
        return Self {
            server: Server::new(),
            database: Database::new(),
            notification: Notification::new(),
            schedule: std::env::var("SCHEDULE")
                .expect("SCHEDULE must be defined")
                .parse()
                .expect("SCHEDULE must be a valid schedule"),
            storage: Storage::new(),
        };
    }
}
