use crate::*;

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
pub enum PurgeReason {
    #[default]
    ActiveStatus,
    LastActive,
    DoubleEmail,
    DoubleName,
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
pub struct PurgeUser {
    pub id: String,
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
pub struct PurgeRobot {}

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
pub struct PurgeScheduler {
    pub last_updated: chrono::DateTime<chrono::Utc>,
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
pub struct PurgeData {
    pub user: PurgeUser,
    pub robot: PurgeRobot,
    pub time: PurgeScheduler,
    pub alert: PurgeScheduler,
    pub reason: Vec<PurgeReason>,
}

#[derive(PartialEq, Eq, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct PurgeUsers {
    users: std::collections::HashMap<String, PurgeData>,
}

impl PurgeUsers {
    pub fn new() -> Self {
        return Self {
            users: std::collections::HashMap::new(),
        };
    }

    pub fn push(
        &mut self,
        user: &models::jira::User,
        robot: &models::robot::Robot,
        reason: PurgeReason,
    ) -> &mut PurgeData {
        return match self.users.entry(user.id.clone()) {
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::hash_map::Entry::Vacant(e) => e.insert(PurgeData {
                user: PurgeUser {
                    id: user.id.clone(),
                },
                robot: PurgeRobot {},
                time: PurgeScheduler {
                    last_updated: robot.scheduler.last_updated,
                },
                alert: PurgeScheduler {
                    last_updated: robot.scheduler.last_updated,
                },
                reason: vec![reason],
            }),
        };
    }
}
