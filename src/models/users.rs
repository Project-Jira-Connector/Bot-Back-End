use super::robots::Robot;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashSet;

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default, Debug, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurgeReason {
    #[default]
    DoubleEmail,
    DoubleName,
    ActiveStatus,
    LastActive,
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationStatus {
    pub invited_at: DateTime<Utc>,
    pub status: String,
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedStatus {
    pub managed: bool,
    pub owner: Option<String>,
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeData {
    pub robot: Robot,
    pub reason: HashSet<PurgeReason>,
    pub time: DateTime<Utc>,
    pub alert: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub platform_role: Option<String>,
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub active: bool,
    pub has_verified_email: bool,
    pub picture: String,
    pub active_status: String,
    pub nickname: String,
    pub title: Option<String>,
    pub timezone: Option<String>,
    pub location: Option<String>,
    pub company_name: Option<String>,
    pub department: Option<String>,
    pub presence: Option<DateTime<Utc>>,
    pub created: DateTime<Utc>,
    pub invitation_status: Option<InvitationStatus>,
    pub system: bool,
    pub org_admin: bool,
    pub site_admin: bool,
    pub sys_admin: bool,
    pub trusted_user: bool,
    pub presence_unavailable: Option<bool>,
    pub managed_status: ManagedStatus,
    pub purge_data: Option<PurgeData>,
}

impl User {
    pub async fn should_delete(&self, now: DateTime<Utc>) -> bool {
        return self.purge_data.as_ref().unwrap().time < now;
    }

    pub async fn should_email(&self, now: DateTime<Utc>) -> bool {
        return self.purge_data.as_ref().unwrap().alert.is_none()
            || self.purge_data.as_ref().unwrap().alert.unwrap() + Duration::days(3) < now;
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub users: Vec<User>,
    pub total: i32,
}
