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
pub struct PurgeReasonsContainer {
    pub data: Vec<PurgeReason>,
}

impl PurgeReasonsContainer {
    pub fn new(reason: PurgeReason) -> Self {
        return Self { data: vec![reason] };
    }

    pub fn push(&mut self, reason: PurgeReason) {
        match self.data.iter().find(|&e| *e == reason) {
            Some(_reason) => {}
            None => {
                self.data.push(reason);
            }
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
pub struct PurgeUser {
    pub user_id: String,
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
pub struct PurgeRobot {
    pub id: mongodb::bson::oid::ObjectId,
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
    pub reasons: PurgeReasonsContainer,
}

#[derive(PartialEq, Eq, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct PurgeUsers {
    pub users: std::collections::HashMap<String, PurgeData>,
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
            std::collections::hash_map::Entry::Occupied(e) => {
                let purge_data = e.into_mut();
                purge_data.reasons.push(reason);
                return purge_data;
            }
            std::collections::hash_map::Entry::Vacant(e) => e.insert(PurgeData {
                user: PurgeUser {
                    user_id: user.id.clone(),
                },
                robot: PurgeRobot { id: robot.id },
                time: PurgeScheduler {
                    last_updated: robot.scheduler.last_updated,
                },
                alert: PurgeScheduler {
                    last_updated: robot.scheduler.last_updated,
                },
                reasons: PurgeReasonsContainer::new(reason),
            }),
        };
    }
}
