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
pub struct Robot {
    #[serde(flatten)]
    pub data: RobotData,
    #[serde(flatten)]
    pub config: RobotConfig,
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
pub struct RobotConfig {
    #[serde(flatten)]
    pub credential: RobotCredential,
    #[serde(flatten)]
    pub scheduler: RobotScheduler,
}

#[serde_with::skip_serializing_none]
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
pub struct RobotData {
    #[serde(flatten)]
    pub id: RobotIdentifier,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
}

#[serde_with::skip_serializing_none]
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
pub struct RobotIdentifier {
    #[serde(rename = "_id")]
    pub unique: Option<mongodb::bson::oid::ObjectId>,
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
#[serde(rename_all = "camelCase")]
pub struct RobotCredential {
    pub platform_email: String,
    pub platform_api_key: String,
    pub cloud_session_token: String,
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
#[serde(rename_all = "camelCase")]
pub struct RobotScheduler {
    pub schedule: i64,
    pub last_active: i64,
    pub check_double_name: bool,
    pub check_double_email: bool,
    pub check_active_status: bool,
}

impl Robot {
    pub fn new(data: RobotData, config: RobotConfig) -> Self {
        return Self { data, config };
    }

    pub fn is_updated(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        if let Some(modified) = self.data.modified {
            if now <= modified + chrono::Duration::days(self.config.scheduler.schedule) {
                return true;
            }
        }
        return false;
    }
}
