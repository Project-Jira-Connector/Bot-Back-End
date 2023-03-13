#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlatformType {
    #[default]
    Cloud,
    Server,
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
pub struct RobotInfo {
    pub name: String,
    pub description: String,
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
pub struct RobotInfoQuery {
    pub name: Option<String>,
    pub description: Option<String>,
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
    pub platform_type: PlatformType,
    pub cloud_session_token: String,
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
#[serde(rename_all = "camelCase")]
pub struct RobotCredentialQuery {
    pub platform_email: Option<String>,
    pub platform_api_key: Option<String>,
    pub platform_type: Option<PlatformType>,
    pub cloud_session_token: Option<String>,
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
#[serde(rename_all = "camelCase")]
pub struct RobotScheduler {
    pub active: bool,
    pub schedule: i64,
    pub last_active: i64,
    pub check_double_name: bool,
    pub check_double_email: bool,
    pub check_active_status: bool,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
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
#[serde(rename_all = "camelCase")]
pub struct RobotSchedulerQuery {
    pub active: Option<bool>,
    pub schedule: Option<i64>,
    pub last_active: Option<i64>,
    pub check_double_name: Option<bool>,
    pub check_double_email: Option<bool>,
    pub check_active_status: Option<bool>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
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
pub struct Robot {
    #[serde(rename = "_id")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(flatten)]
    pub info: RobotInfo,
    #[serde(flatten)]
    pub credential: RobotCredential,
    #[serde(flatten)]
    pub scheduler: RobotScheduler,
}

impl Robot {
    pub fn is_up_to_date(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        if let Some(last_updated) = self.scheduler.last_updated {
            if now <= last_updated + chrono::Duration::days(self.scheduler.schedule) {
                return true;
            }
        }
        return false;
    }
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
pub struct RobotQuery {
    #[serde(rename = "_id")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(flatten)]
    pub info: RobotInfoQuery,
    #[serde(flatten)]
    pub credential: RobotCredentialQuery,
    #[serde(flatten)]
    pub scheduler: RobotSchedulerQuery,
}

impl From<&mut Robot> for RobotQuery {
    fn from(robot: &mut Robot) -> Self {
        return RobotQuery {
            id: robot.id,
            info: RobotInfoQuery {
                name: Some(robot.info.name.clone()),
                description: Some(robot.info.description.clone()),
            },
            credential: RobotCredentialQuery {
                platform_email: Some(robot.credential.platform_email.clone()),
                platform_api_key: Some(robot.credential.platform_api_key.clone()),
                platform_type: Some(robot.credential.platform_type),
                cloud_session_token: Some(robot.credential.cloud_session_token.clone()),
            },
            scheduler: RobotSchedulerQuery {
                active: Some(robot.scheduler.active),
                schedule: Some(robot.scheduler.schedule),
                last_active: Some(robot.scheduler.last_active),
                check_double_name: Some(robot.scheduler.check_double_name),
                check_double_email: Some(robot.scheduler.check_double_email),
                check_active_status: Some(robot.scheduler.check_active_status),
                last_updated: robot.scheduler.last_updated,
            },
        };
    }
}
