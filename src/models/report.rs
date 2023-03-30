use crate::*;

#[derive(PartialEq, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub queued: Vec<models::purge::PurgeData>,
    pub removed: Vec<models::purge::PurgeLog>,
}

impl Report {
    pub fn new(
        queued: Vec<models::purge::PurgeData>,
        removed: Vec<models::purge::PurgeLog>,
    ) -> Self {
        return Self { queued, removed };
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
pub struct Generator {
    #[serde(flatten)]
    pub robot_id: models::robot::RobotIdentifier,
    pub email: String,
}
